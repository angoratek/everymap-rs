use crate::client::{endpoint_url, GoogleClient};
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

/// Service root of the Google Static Maps API.
const STATIC_MAPS_SERVICE_ROOT: &str = "https://maps.googleapis.com";

/// Documented path of the Google Static Maps API.
const STATIC_MAPS_PATH: &str = "/maps/api/staticmap";

/// Implementation of MapImageProvider for Google Static Maps API.
pub struct GoogleMapImageProvider {
    pub(crate) client: Arc<GoogleClient>,
    pub(crate) service_root: String,
}

impl GoogleMapImageProvider {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            service_root: STATIC_MAPS_SERVICE_ROOT.to_string(),
        }
    }

    /// Creates an image provider against a Google service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    /// - `service_root` — the scheme and host that serve the API, for example
    ///   `https://maps.googleapis.com` or a test server's origin. The
    ///   documented path is appended.
    pub fn with_base_url(client: Arc<GoogleClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
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

        let url = endpoint_url(&self.service_root, STATIC_MAPS_PATH);
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;

        let content_type = response
            .headers()
            .get("content-type")
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

        let data = response.into_body();

        Ok(ImageResponse { data, content_type })
    }
}
