use async_trait::async_trait;
use everymap_core::domains::tiling::{TileProvider, TileOptions, TileResponse};
use everymap_core::error::EveryMapResult;
use crate::client::MapBoxClient;
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of TileProvider for MapBox Vector/Raster Tiles API.
pub struct MapBoxTileProvider {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxTileProvider {
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
impl TileProvider for MapBoxTileProvider {
    async fn get_tile(&self, z: u32, x: u32, y: u32, options: &TileOptions) -> EveryMapResult<TileResponse> {
        let tileset_id = options.provider_extra.as_ref()
            .and_then(|e| e.get("tileset_id")).and_then(|v| v.as_str()).unwrap_or("mapbox.mapbox-streets-v8");
        let format = options.format.as_deref().unwrap_or("mvt");

        let url = format!("{}/v4/{}/{}/{}/{}.{}", self.base_url, tileset_id, z, x, y, format);

        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let response = self.client.request(builder).await?;

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

        let data = response.bytes().await?.to_vec();

        Ok(TileResponse { data, content_type })
    }
}