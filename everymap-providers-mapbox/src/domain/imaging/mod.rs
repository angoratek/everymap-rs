use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::imaging::{ImageOptions, ImageResponse, MapImageProvider};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of MapImageProvider for MapBox Static Images API.
pub struct MapBoxMapImageProvider {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxMapImageProvider {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: MAP_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl MapImageProvider for MapBoxMapImageProvider {
    async fn get_image(
        &self,
        center: &Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse> {
        let style = options
            .provider_extra
            .as_ref()
            .and_then(|e| e.get("style"))
            .and_then(|v| v.as_str())
            .unwrap_or("mapbox/streets-v12");

        // Core width/height override the size argument when set
        let width = options.width.unwrap_or(size.0);
        let height = options.height.unwrap_or(size.1);

        // Build the full style URL: /styles/v1/{username}/{style_id}/static/{lon},{lat},{zoom}/{width}x{height}@2x
        let url = format!(
            "{}/styles/v1/{}/static/{},{},{}/{}x{}@2x",
            self.base_url, style, center.lng, center.lat, zoom, width, height
        );

        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if options.format.is_some() {
            log::warn!(
                "MapBox Static Images API does not support a format parameter; \
                 format will be ignored (MapBox always returns PNG)"
            );
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
