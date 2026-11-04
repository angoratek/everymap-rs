//! Raster tiles from the MapQuest Tiles API.
//!
//! The provider implements
//! [`RasterTileProvider`]
//! against the documented rendered-tile path
//! `tiles/v1/render/latest/{style}/{z}/{x}/{y}/{size}/{format}`. The request's
//! image format selects `png` or `jpeg`, and its resolution selects the plain
//! `256` size segment or the double-texel `256@2x` form. Every returned
//! [`RasterTileData`] echoes
//! the request's format and resolution, so a cache key or renderer never has
//! to re-derive what the bytes are.

use crate::client::MapQuestClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileData, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use std::sync::Arc;

/// Default map style served to the renderer.
///
/// The Tiles API also documents the `night` and `satellite` styles; use
/// [`MapQuestRasterTileProvider::with_tile_style`] to select one.
pub const DEFAULT_MAPQUEST_TILE_STYLE: &str = "vivid";

/// Path prefix of the rendered-tile endpoint.
const TILE_PATH_PREFIX: &str = "tiles/v1/render/latest";

/// Size path segment of the standard-resolution request.
const STANDARD_TILE_SIZE_PATH_SEGMENT: &str = "256";

/// Size path segment of the retina-resolution request: twice the texels per
/// edge for the same logical tile span.
const RETINA_TILE_SIZE_PATH_SEGMENT: &str = "256@2x";

/// Deepest zoom level the Tiles API serves.
const MAXIMUM_TILE_ZOOM_LEVEL: u8 = 20;

/// Raster tiles served from the MapQuest Tiles API.
///
/// Construct one with [`Self::new`] for the default style, or
/// [`Self::with_tile_style`] for another documented style.
pub struct MapQuestRasterTileProvider {
    client: Arc<MapQuestClient>,
    tile_style: String,
}

impl MapQuestRasterTileProvider {
    /// Builds a tile provider serving the default style.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A provider serving [`DEFAULT_MAPQUEST_TILE_STYLE`].
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self::with_tile_style(client, DEFAULT_MAPQUEST_TILE_STYLE.to_string())
    }

    /// Builds a tile provider serving an explicit style.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    /// - `tile_style` — documented style segment, such as `vivid`, `night` or
    ///   `satellite`.
    ///
    /// # Returns
    /// A provider serving `tile_style`.
    pub fn with_tile_style(client: Arc<MapQuestClient>, tile_style: String) -> Self {
        Self { client, tile_style }
    }

    /// The style this provider serves.
    pub fn tile_style(&self) -> &str {
        &self.tile_style
    }
}

#[async_trait]
impl RasterTileProvider for MapQuestRasterTileProvider {
    /// Fetches one web-mercator XYZ raster tile.
    ///
    /// # Parameters
    /// - `request` — tile coordinates, image format and resolution.
    ///
    /// # Returns
    /// The encoded tile bytes together with the request's format and
    /// resolution.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when the zoom level exceeds the
    ///   service maximum or a tile index is outside the zoom level's range.
    /// - The shared client's HTTP, rate-limit or transport error when the tile
    ///   cannot be fetched.
    async fn fetch_raster_tile(
        &self,
        request: &RasterTileRequest,
    ) -> EveryMapResult<RasterTileData> {
        if request.zoom_level > MAXIMUM_TILE_ZOOM_LEVEL {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {} exceeds the documented Tiles API maximum of {MAXIMUM_TILE_ZOOM_LEVEL}",
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
        let path = format!(
            "{TILE_PATH_PREFIX}/{}/{}/{}/{}/{}/{}",
            self.tile_style,
            request.zoom_level,
            request.tile_x,
            request.tile_y,
            tile_size_path_segment(request.tile_resolution),
            image_format_path_segment(request.image_format),
        );
        let image_bytes = self.client.get_bytes(&path).await?;
        Ok(RasterTileData::for_request(request, image_bytes))
    }
}

/// Size path segment the Tiles API expects for a resolution.
fn tile_size_path_segment(tile_resolution: RasterTileResolution) -> &'static str {
    match tile_resolution {
        RasterTileResolution::Standard => STANDARD_TILE_SIZE_PATH_SEGMENT,
        RasterTileResolution::Retina => RETINA_TILE_SIZE_PATH_SEGMENT,
    }
}

/// Format path segment the Tiles API expects for an image format.
fn image_format_path_segment(image_format: RasterImageFormat) -> &'static str {
    match image_format {
        RasterImageFormat::Png => "png",
        RasterImageFormat::Jpeg => "jpeg",
    }
}

#[cfg(test)]
mod tests {
    use super::{image_format_path_segment, tile_size_path_segment};
    use everymap_core::domains::tiling::{RasterImageFormat, RasterTileResolution};

    #[test]
    fn image_formats_map_to_the_documented_path_segments() {
        assert_eq!(image_format_path_segment(RasterImageFormat::Png), "png");
        assert_eq!(image_format_path_segment(RasterImageFormat::Jpeg), "jpeg");
    }

    #[test]
    fn resolutions_map_to_the_documented_size_segments() {
        assert_eq!(
            tile_size_path_segment(RasterTileResolution::Standard),
            "256"
        );
        assert_eq!(
            tile_size_path_segment(RasterTileResolution::Retina),
            "256@2x"
        );
    }
}
