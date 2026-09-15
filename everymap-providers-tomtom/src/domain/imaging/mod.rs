use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of MapImageProvider for TomTom Map Display API (static image).
pub struct TomTomMapImageProvider {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomMapImageProvider {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: MAP_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl MapImageProvider for TomTomMapImageProvider {
    async fn get_image(
        &self,
        center: &Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse> {
        let url = format!("{}/map/1/staticimage", self.base_url);

        // Core width/height override the size argument when set
        let width = options.width.unwrap_or(size.0);
        let height = options.height.unwrap_or(size.1);

        let mut params: Vec<(&str, String)> = vec![
            ("center", format!("{},{}", center.lng, center.lat)),
            ("zoom", zoom.to_string()),
            ("width", width.to_string()),
            ("height", height.to_string()),
        ];

        if let Some(fmt) = &options.format {
            let format_val = match fmt.as_str() {
                "jpg" | "jpeg" => "jpg",
                "gif" => "gif",
                _ => "png",
            };
            params.push(("format", format_val.to_string()));
        }

        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }

        // Extract TomTom-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("layer").and_then(|v| v.as_str()) {
                    params.push(("layer", v.to_string()));
                }
                if let Some(v) = obj.get("style").and_then(|v| v.as_str()) {
                    params.push(("style", v.to_string()));
                }
            }
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
