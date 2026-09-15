use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

const STATIC_MAPS_BASE_URL: &str = "https://maps.googleapis.com/maps/api/staticmap";

/// Implementation of MapImageProvider for Google Static Maps API.
pub struct GoogleMapImageProvider {
    pub(crate) client: Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleMapImageProvider {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: STATIC_MAPS_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl MapImageProvider for GoogleMapImageProvider {
    async fn get_image(
        &self,
        center: &Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse> {
        // Core width/height override the size argument when set
        let width = options.width.unwrap_or(size.0);
        let height = options.height.unwrap_or(size.1);

        let mut params: Vec<(String, String)> = vec![
            (
                "center".to_string(),
                format!("{},{}", center.lat, center.lng),
            ),
            ("zoom".to_string(), zoom.to_string()),
            ("size".to_string(), format!("{}x{}", width, height)),
        ];

        // Format: png (default), jpg, gif
        if let Some(fmt) = &options.format {
            let format_val = match fmt.as_str() {
                "jpg" | "jpeg" => "jpg",
                "gif" => "gif",
                _ => "png",
            };
            params.push(("format".to_string(), format_val.to_string()));
        }

        if let Some(lang) = &options.language {
            params.push(("language".to_string(), lang.clone()));
        }

        // Extract Google-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("maptype").and_then(|v| v.as_str()) {
                    params.push(("maptype".to_string(), v.to_string()));
                }
                if let Some(v) = obj.get("scale").and_then(|v| v.as_u64()) {
                    params.push(("scale".to_string(), v.to_string()));
                }
                if let Some(v) = obj.get("markers").and_then(|v| v.as_str()) {
                    params.push(("markers".to_string(), v.to_string()));
                }
                if let Some(v) = obj.get("path").and_then(|v| v.as_str()) {
                    params.push(("path".to_string(), v.to_string()));
                }
                if let Some(v) = obj.get("visible").and_then(|v| v.as_str()) {
                    params.push(("visible".to_string(), v.to_string()));
                }
                if let Some(v) = obj.get("style").and_then(|v| v.as_str()) {
                    params.push(("style".to_string(), v.to_string()));
                }
            }
        }

        let url = self.base_url.clone();
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
