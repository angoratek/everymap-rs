use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileData, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use std::sync::Arc;

const MAP_BASE_URL: &str = "https://api.tomtom.com";

/// The Map Display layer the typed contract serves when the caller names none.
pub const DEFAULT_TOMTOM_TILE_LAYER: &str = "basic";

/// The Map Display style the typed contract serves when the caller names none.
pub const DEFAULT_TOMTOM_TILE_STYLE: &str = "main";

/// Deepest zoom the Map Display API serves.
pub const TOMTOM_RASTER_MAXIMUM_ZOOM_LEVEL: u8 = 22;

/// TomTom raster tiles served through the typed [`RasterTileProvider`] contract.
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
impl RasterTileProvider for TomTomTileProvider {
    /// Fetches one PNG tile of the default Map Display layer and style.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the request asks for
    /// JPEG or a retina resolution, both of which the Map Display tile
    /// endpoint does not serve, and when the zoom level or tile index is out
    /// of range.
    async fn fetch_raster_tile(
        &self,
        request: &RasterTileRequest,
    ) -> EveryMapResult<RasterTileData> {
        if request.image_format != RasterImageFormat::Png {
            return Err(EveryMapError::ValidationError(
                "the Map Display tile endpoint serves PNG imagery only".to_string(),
            ));
        }
        if request.tile_resolution != RasterTileResolution::Standard {
            return Err(EveryMapError::ValidationError(
                "the Map Display tile endpoint serves standard-resolution tiles only".to_string(),
            ));
        }
        if request.zoom_level > TOMTOM_RASTER_MAXIMUM_ZOOM_LEVEL {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {} exceeds the documented Map Display maximum of \
                 {TOMTOM_RASTER_MAXIMUM_ZOOM_LEVEL}",
                request.zoom_level
            )));
        }
        let maximum_tile_index = (1_u32 << request.zoom_level) - 1;
        if request.tile_x > maximum_tile_index || request.tile_y > maximum_tile_index {
            return Err(EveryMapError::ValidationError(format!(
                "tile x={} y={} exceeds the {maximum_tile_index} maximum at zoom {}",
                request.tile_x, request.tile_y, request.zoom_level
            )));
        }
        let url = format!(
            "{}/map/1/tile/{}/{}/{}/{}/{}.png",
            self.base_url,
            DEFAULT_TOMTOM_TILE_LAYER,
            DEFAULT_TOMTOM_TILE_STYLE,
            request.zoom_level,
            request.tile_x,
            request.tile_y
        );
        let http_request = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&[("tileSize", "256".to_string())]);
        let response = self.client.request(http_request).await?;
        Ok(RasterTileData::for_request(request, response.into_body()))
    }
}
