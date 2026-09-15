pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

pub use types::*;

const IMAGING_BASE_URL: &str = "https://maps.hereapi.com/mia/v3";

/// Options for HERE Map Image API v3.
#[derive(Debug, Clone, Default)]
pub struct HereImageOptions {
    /// Image format (default: PNG).
    pub format: ImageFormat,
    /// Map style (default: default).
    pub style: Option<String>,
    /// Language for labels (e.g., "en-US").
    pub lang: Option<String>,
    /// Political view for disputed borders (ISO 3166-1 alpha-3).
    pub political_view: Option<String>,
    /// Include POI markers on the map.
    pub poi: Option<String>,
    /// Background color (hex, e.g., "FFFFFF").
    pub bg: Option<String>,
    /// Center marker (adds a marker at the center).
    pub center_marker: Option<bool>,
    /// Overlay data (custom shapes/lines on the map).
    pub overlay: Option<String>,
}

/// Implementation of MapImageProvider for HERE Technologies.
pub struct HereMapImageProvider {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereMapImageProvider {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: IMAGING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core `ImageOptions` to HERE-specific `HereImageOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn image_options_from_core(options: &ImageOptions) -> HereImageOptions {
    let mut here_options = HereImageOptions {
        lang: options.language.clone(),
        ..Default::default()
    };

    // Read format from core field first, then provider_extra as override
    if let Some(fmt) = &options.format {
        here_options.format = match fmt.to_lowercase().as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpg,
            "gif" => ImageFormat::Gif,
            "bmp" => ImageFormat::Bmp,
            "svg" => ImageFormat::Svg,
            "png8" => ImageFormat::Png8,
            "png32" => ImageFormat::Png32,
            _ => ImageFormat::Png,
        };
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("format").and_then(|v| v.as_str()) {
                here_options.format = match v {
                    "jpg" => ImageFormat::Jpg,
                    "gif" => ImageFormat::Gif,
                    "bmp" => ImageFormat::Bmp,
                    "svg" => ImageFormat::Svg,
                    "png8" => ImageFormat::Png8,
                    "png32" => ImageFormat::Png32,
                    _ => ImageFormat::Png,
                };
            }
            if let Some(v) = obj.get("style").and_then(|v| v.as_str()) {
                here_options.style = Some(v.to_string());
            }
            if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
                here_options.political_view = Some(v.to_string());
            }
            if let Some(v) = obj.get("poi").and_then(|v| v.as_str()) {
                here_options.poi = Some(v.to_string());
            }
            if let Some(v) = obj.get("bg").and_then(|v| v.as_str()) {
                here_options.bg = Some(v.to_string());
            }
            if let Some(v) = obj.get("center_marker").and_then(|v| v.as_bool()) {
                here_options.center_marker = Some(v);
            }
            if let Some(v) = obj.get("overlay").and_then(|v| v.as_str()) {
                here_options.overlay = Some(v.to_string());
            }
        }
    }

    here_options
}

#[async_trait]
impl MapImageProvider for HereMapImageProvider {
    async fn get_image(
        &self,
        center: &everymap_core::types::Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse> {
        let here_options = image_options_from_core(options);

        // Core width/height override the size argument when set
        let width = options.width.unwrap_or(size.0);
        let height = options.height.unwrap_or(size.1);

        let format_ext = match &here_options.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Svg => "svg",
            ImageFormat::Png8 => "png8",
            ImageFormat::Png32 => "png32",
        };

        let url = format!(
            "{}/base/mc/center:{},{};zoom={}/{}x{}/{}",
            self.base_url, center.lat, center.lng, zoom, width, height, format_ext
        );

        let mut params: Vec<(String, String)> = vec![];

        if let Some(style) = &here_options.style {
            params.push(("style".to_string(), style.clone()));
        }
        if let Some(lang) = &here_options.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(political_view) = &here_options.political_view {
            params.push(("politicalView".to_string(), political_view.clone()));
        }
        if let Some(poi) = &here_options.poi {
            params.push(("poi".to_string(), poi.clone()));
        }
        if let Some(bg) = &here_options.bg {
            params.push(("bg".to_string(), bg.clone()));
        }
        if let Some(overlay) = &here_options.overlay {
            params.push(("overlay".to_string(), overlay.clone()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

        let data = response.bytes().await?.to_vec();

        Ok(ImageResponse { data, content_type })
    }
}
