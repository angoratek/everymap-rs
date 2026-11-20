//! Static map images from the MapQuest Static Map API.
//!
//! The provider implements [`MapImageProvider`] against the documented
//! `staticmap/v5/map` endpoint. The center is requested as `lat,lng`, the size
//! combines the core's width and height overrides with the caller's size, and
//! a JSON error envelope is translated into the shared error model instead of
//! being returned as image bytes.

use crate::client::MapQuestClient;
use crate::error_mapping::{invalid_response, validate_info_envelope};
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::Method;
use everymap_core::types::Coordinate;
use std::sync::Arc;

/// Service path of the Static Map API.
const STATIC_MAP_PATH: &str = "staticmap/v5/map";

/// Largest image edge the Static Map API serves, in pixels.
pub const MAXIMUM_IMAGE_DIMENSION: u32 = 1920;

/// Deepest zoom level the Static Map API serves.
pub const MAXIMUM_IMAGE_ZOOM_LEVEL: u32 = 20;

/// Implementation of `MapImageProvider` for the MapQuest Static Map API.
pub struct MapQuestMapImageProvider {
    client: Arc<MapQuestClient>,
}

impl MapQuestMapImageProvider {
    /// Builds a static-image provider around a shared client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

/// Maps a requested format onto the Static Map API's documented value.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] for a format the endpoint does
/// not document.
fn image_format_parameter(format: Option<&str>) -> EveryMapResult<&'static str> {
    match format {
        None | Some("png") => Ok("png"),
        Some("jpg") | Some("jpeg") => Ok("jpg"),
        Some("gif") => Ok("gif"),
        Some(other) => Err(EveryMapError::ValidationError(format!(
            "the MapQuest Static Map API does not serve the '{other}' format; \
             use png, jpg or gif"
        ))),
    }
}

/// The content type's media type without parameters.
fn media_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_string()
}

#[async_trait]
impl MapImageProvider for MapQuestMapImageProvider {
    async fn get_image(
        &self,
        center: &Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse> {
        let width = options.width.unwrap_or(size.0);
        let height = options.height.unwrap_or(size.1);
        if width == 0 || height == 0 {
            return Err(EveryMapError::ValidationError(
                "the MapQuest Static Map API requires a positive image width and height"
                    .to_string(),
            ));
        }
        if width > MAXIMUM_IMAGE_DIMENSION || height > MAXIMUM_IMAGE_DIMENSION {
            return Err(EveryMapError::ValidationError(format!(
                "the MapQuest Static Map API serves at most {MAXIMUM_IMAGE_DIMENSION} pixels per edge"
            )));
        }
        if zoom > MAXIMUM_IMAGE_ZOOM_LEVEL {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {zoom} exceeds the documented Static Map API maximum of \
                 {MAXIMUM_IMAGE_ZOOM_LEVEL}"
            )));
        }

        let image_format = image_format_parameter(options.format.as_deref())?;
        let mut query: Vec<(String, String)> = vec![
            (
                "center".to_string(),
                format!("{},{}", center.lat, center.lng),
            ),
            ("zoom".to_string(), zoom.to_string()),
            ("size".to_string(), format!("{width},{height}")),
            ("format".to_string(), image_format.to_string()),
        ];

        if options.language.is_some() {
            log::warn!(
                "the MapQuest Static Map API does not document a language parameter; \
                 language will be ignored"
            );
        }

        if let Some(extra) = &options.provider_extra {
            if let Some(object) = extra.as_object() {
                for (name, value) in object {
                    let rendered = match value {
                        serde_json::Value::String(text) => Some(text.clone()),
                        serde_json::Value::Number(number) => Some(number.to_string()),
                        serde_json::Value::Bool(flag) => Some(flag.to_string()),
                        _ => None,
                    };
                    match rendered {
                        Some(rendered) => query.push((name.clone(), rendered)),
                        None => log::warn!(
                            "MapQuest static-image option '{name}' is not a string, number or \
                             boolean; it will be ignored"
                        ),
                    }
                }
            }
        }

        let request = self
            .client
            .build_request(Method::Get, STATIC_MAP_PATH, &query);
        let response = self.client.request(request).await?;
        let content_type = response.headers().get("content-type").map(media_type);
        let body = response.into_body();

        if content_type
            .as_deref()
            .is_some_and(|media_type| media_type == "application/json")
        {
            let payload: serde_json::Value = serde_json::from_slice(&body)?;
            validate_info_envelope(&payload)?;
            return Err(invalid_response(
                "the Static Map API answered with JSON instead of an image",
            ));
        }

        Ok(ImageResponse::new(body, content_type))
    }
}

#[cfg(test)]
mod tests {
    use super::{image_format_parameter, media_type, MAXIMUM_IMAGE_DIMENSION};
    use everymap_core::error::EveryMapError;

    #[test]
    fn documented_formats_map_to_the_service_values() {
        assert_eq!(image_format_parameter(None).unwrap(), "png");
        assert_eq!(image_format_parameter(Some("png")).unwrap(), "png");
        assert_eq!(image_format_parameter(Some("jpeg")).unwrap(), "jpg");
        assert_eq!(image_format_parameter(Some("gif")).unwrap(), "gif");
    }

    #[test]
    fn an_undocumented_format_is_refused() {
        assert!(matches!(
            image_format_parameter(Some("webp")),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn a_content_type_loses_its_parameters() {
        assert_eq!(media_type("image/png; charset=binary"), "image/png");
    }

    #[test]
    fn the_documented_maximum_is_a_named_constant() {
        assert_eq!(MAXIMUM_IMAGE_DIMENSION, 1920);
    }
}
