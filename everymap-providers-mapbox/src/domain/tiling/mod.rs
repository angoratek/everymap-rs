use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{
    VectorTileData, VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.mapbox.com";

/// The tileset the typed contract serves when the caller names none: MapBox's
/// Streets v8, the tileset the built-in styles are authored against.
pub const DEFAULT_MAPBOX_TILESET_ID: &str = "mapbox.mapbox-streets-v8";

/// Attribution MapBox imagery requires wherever it is drawn.
pub const MAPBOX_VECTOR_ATTRIBUTION: &str = "© Mapbox © OpenStreetMap";

/// Deepest zoom the Vector Tiles API serves.
pub const MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL: u8 = 22;

/// MapBox vector tiles served through the typed [`VectorTileProvider`] contract.
pub struct MapBoxTileProvider {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
    /// The tileset the provider serves; [`DEFAULT_MAPBOX_TILESET_ID`] unless a
    /// caller selects another with [`Self::with_tileset_id`].
    pub(crate) tileset_id: String,
}

impl MapBoxTileProvider {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: MAP_BASE_URL.to_string(),
            tileset_id: DEFAULT_MAPBOX_TILESET_ID.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self {
            client,
            base_url,
            tileset_id: DEFAULT_MAPBOX_TILESET_ID.to_string(),
        }
    }

    /// Selects the tileset this provider serves, replacing the default
    /// Streets v8 set. The tileset id is the MapBox Tiles API account and
    /// tileset pair, for example `mapbox.mapbox-streets-v8`.
    pub fn with_tileset_id(mut self, tileset_id: impl Into<String>) -> Self {
        self.tileset_id = tileset_id.into();
        self
    }
}

#[async_trait]
impl VectorTileProvider for MapBoxTileProvider {
    /// Describes the served tileset: the default Streets v8 set, or the one
    /// the caller selected with [`MapBoxTileProvider::with_tileset_id`].
    fn source_descriptor(&self) -> VectorTileSourceDescriptor {
        VectorTileSourceDescriptor {
            source_name: format!("mapbox/{}", self.tileset_id),
            attribution_text: MAPBOX_VECTOR_ATTRIBUTION.to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["streets".to_string()],
        }
    }

    /// Fetches one MVT tile of the served tileset.
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
        if zoom_level > MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {zoom_level} exceeds the documented vector-tiles maximum of \
                 {MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL}"
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
            "{}/v4/{}/{}/{}/{}.mvt",
            self.base_url, self.tileset_id, zoom_level, tile_x, tile_y
        );
        let request = self
            .client
            .build_request(everymap_core::http::Method::GET, &url);
        let response = self.client.request(request).await?;
        VectorTileData::from_wire_bytes(tile_x, tile_y, zoom_level, response.into_body())
    }
}
