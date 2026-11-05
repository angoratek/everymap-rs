pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{
    VectorTileData, VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use std::sync::Arc;

pub use types::*;

const TILING_BASE_URL: &str = "https://vector.hereapi.com/v2";

/// Options for HERE Vector Tile API v2.
#[derive(Debug, Clone, Default)]
pub struct HereTileOptions {
    /// The tile layer to request.
    pub layer: TileLayer,
    /// Tile format (default: OmnichannelVector / .omv).
    pub format: TileFormat,
    /// Political view for disputed borders (ISO 3166-1 alpha-3 country code).
    pub political_view: Option<String>,
}

/// Deepest zoom the HERE Vector Tile API v2 serves.
///
/// The service answers any deeper request with a validation error naming the
/// accepted range.
pub const HERE_VECTOR_MAXIMUM_ZOOM_LEVEL: u8 = 17;

/// Attribution HERE map imagery requires wherever it is drawn.
pub const HERE_VECTOR_ATTRIBUTION: &str = "© HERE — map data © OpenStreetMap contributors (ODbL)";

/// HERE vector tiles served through the typed [`VectorTileProvider`] contract.
///
/// The service's only output segment is `omv`; its bytes are Mapbox Vector
/// Tile protobuf, verified by decoding a fetched tile, so the typed contract
/// carries them unchanged.
pub struct HereTileProvider {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereTileProvider {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: TILING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl VectorTileProvider for HereTileProvider {
    /// Describes the HERE base tileset: its cache namespace, attribution and
    /// zoom span.
    fn source_descriptor(&self) -> VectorTileSourceDescriptor {
        VectorTileSourceDescriptor {
            source_name: "here/vector".to_string(),
            attribution_text: HERE_VECTOR_ATTRIBUTION.to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: HERE_VECTOR_MAXIMUM_ZOOM_LEVEL,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["omv".to_string(), "base".to_string()],
        }
    }

    /// Fetches one tile of the base layer.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the zoom level is past
    /// the documented maximum or the tile index is outside the zoom level's
    /// range.
    async fn fetch_vector_tile(
        &self,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<VectorTileData> {
        self.fetch_vector_tile_in_layer("base", tile_x, tile_y, zoom_level)
            .await
    }
}

impl HereTileProvider {
    /// Fetches one tile of a named HERE layer.
    ///
    /// # Parameters
    /// - `layer` — the style layer (`base`, `core`, `hybrid`, `mapbox`).
    /// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
    /// - `zoom_level` — the web-mercator zoom level.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the zoom level or the
    /// tile index is outside the served range.
    pub async fn fetch_vector_tile_in_layer(
        &self,
        layer: &str,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<VectorTileData> {
        if zoom_level > HERE_VECTOR_MAXIMUM_ZOOM_LEVEL {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {zoom_level} exceeds the documented vector-tiles maximum of \
                 {HERE_VECTOR_MAXIMUM_ZOOM_LEVEL}"
            )));
        }
        let maximum_tile_index = (1_u32 << zoom_level) - 1;
        if tile_x > maximum_tile_index || tile_y > maximum_tile_index {
            return Err(EveryMapError::ValidationError(format!(
                "tile x={tile_x} y={tile_y} exceeds the {maximum_tile_index} maximum at zoom \
                 {zoom_level}"
            )));
        }
        let url = format!(
            "{}/vectortiles/{}/mc/{}/{}/{}/omv",
            self.base_url, layer, zoom_level, tile_x, tile_y
        );
        let request = self
            .client
            .build_request(everymap_core::http::Method::GET, &url);
        let response = self.client.request(request).await?;
        VectorTileData::from_wire_bytes(tile_x, tile_y, zoom_level, response.into_body())
    }
}
