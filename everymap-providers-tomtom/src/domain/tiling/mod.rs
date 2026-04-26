use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{TileOptions, TileProvider, TileResponse};
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of TileProvider for TomTom Map Display API.
pub struct TomTomTileProvider {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomTileProvider {
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
impl TileProvider for TomTomTileProvider {
    async fn get_tile(
        &self,
        z: u32,
        x: u32,
        y: u32,
        options: &TileOptions,
    ) -> EveryMapResult<TileResponse> {
        let layer = options
            .provider_extra
            .as_ref()
            .and_then(|e| e.get("layer"))
            .and_then(|v| v.as_str())
            .unwrap_or("basic");
        let style = options
            .provider_extra
            .as_ref()
            .and_then(|e| e.get("style"))
            .and_then(|v| v.as_str())
            .unwrap_or("main");

        let ext = match options.format.as_deref() {
            Some("jpg") | Some("jpeg") => "jpg",
            _ => "png",
        };
        let url = format!(
            "{}/map/1/tile/{}/{}/{}/{}/{}.{}",
            self.base_url, layer, style, z, x, y, ext
        );

        let params: Vec<(&str, String)> = vec![("tileSize", "256".to_string())];

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

        Ok(TileResponse { data, content_type })
    }
}
