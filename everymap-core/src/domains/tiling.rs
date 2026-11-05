use crate::error::{EveryMapError, EveryMapResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Encoded image format a raster tile provider delivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RasterImageFormat {
    /// PNG imagery.
    Png,
    /// JPEG imagery.
    Jpeg,
}

impl RasterImageFormat {
    /// Conventional file extension for this encoding (`png`, `jpg`).
    ///
    /// Providers that build a URL from the format use this rather than
    /// spelling the extension themselves.
    pub fn file_extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// Pixel resolution a raster tile is rendered at.
///
/// `Standard` is the plain tile form. `Retina` has twice the texels per edge
/// for the same logical tile span, which is the `@2x` request form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RasterTileResolution {
    /// One texel per logical tile pixel.
    #[default]
    Standard,
    /// Twice the texels per logical tile pixel (the `@2x` request form).
    Retina,
}

impl RasterTileResolution {
    /// Query-parameter value providers send to request this resolution.
    pub fn query_value(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Retina => "retina",
        }
    }
}

/// One typed web-mercator XYZ raster tile request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterTileRequest {
    /// Tile column coordinate at the given zoom level.
    pub tile_x: u32,
    /// Tile row coordinate at the given zoom level.
    pub tile_y: u32,
    /// Zoom level this tile belongs to.
    pub zoom_level: u8,
    /// Image encoding the caller needs.
    pub image_format: RasterImageFormat,
    /// Pixel resolution the caller needs.
    pub tile_resolution: RasterTileResolution,
}

impl RasterTileRequest {
    /// Builds a request for the tile at `tile_x`/`tile_y`/`zoom_level`,
    /// encoded as `image_format` at `tile_resolution`.
    pub fn new(
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
        image_format: RasterImageFormat,
        tile_resolution: RasterTileResolution,
    ) -> Self {
        Self {
            tile_x,
            tile_y,
            zoom_level,
            image_format,
            tile_resolution,
        }
    }
}

/// One fetched web-mercator XYZ raster tile.
///
/// The format and resolution echo the request that produced the tile, so a
/// cache key or a renderer never has to re-derive what the bytes are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterTileData {
    /// Tile column coordinate at the given zoom level.
    pub tile_x: u32,
    /// Tile row coordinate at the given zoom level.
    pub tile_y: u32,
    /// Zoom level this tile belongs to.
    pub zoom_level: u8,
    /// Encoding of the byte payload.
    pub image_format: RasterImageFormat,
    /// Pixel resolution the payload was rendered at.
    ///
    /// A serialized payload that predates this field deserializes as
    /// [`RasterTileResolution::Standard`].
    #[serde(default)]
    pub tile_resolution: RasterTileResolution,
    /// Encoded image bytes.
    pub image_bytes: Vec<u8>,
}

impl RasterTileData {
    /// Builds a response that echoes every coordinate, format and resolution
    /// field of the request that produced it.
    pub fn for_request(request: &RasterTileRequest, image_bytes: Vec<u8>) -> Self {
        Self {
            tile_x: request.tile_x,
            tile_y: request.tile_y,
            zoom_level: request.zoom_level,
            image_format: request.image_format,
            tile_resolution: request.tile_resolution,
            image_bytes,
        }
    }
}

/// Raster map imagery served as encoded image tiles.
///
/// A provider that cannot serve the requested format or resolution returns an
/// error instead of silently re-encoding the payload.
#[async_trait]
pub trait RasterTileProvider: Send + Sync {
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
    /// Returns [`EveryMapError::HttpError`] or
    /// [`EveryMapError::TransportError`] when the request cannot be
    /// completed, and [`EveryMapError::ValidationError`] when the provider
    /// does not serve the requested format or resolution.
    async fn fetch_raster_tile(
        &self,
        request: &RasterTileRequest,
    ) -> EveryMapResult<RasterTileData>;
}

/// First byte of the two-byte gzip magic number (`1f 8b`).
const GZIP_MAGIC_FIRST_BYTE: u8 = 0x1f;
/// Second byte of the two-byte gzip magic number (`1f 8b`).
const GZIP_MAGIC_SECOND_BYTE: u8 = 0x8b;

/// Wire encoding of a vector tile's byte payload.
///
/// Tile services commonly serve Mapbox Vector Tile protobuf with
/// `Content-Encoding: gzip`. The record preserves what arrived on the wire; a
/// decoder applies gzip decompression only for
/// [`VectorTileEncoding::GzippedMapboxVectorTile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorTileEncoding {
    /// Plain MVT protobuf bytes.
    MapboxVectorTile,
    /// MVT protobuf bytes wrapped in gzip (the CDN's `Content-Encoding`).
    GzippedMapboxVectorTile,
}

/// One fetched web-mercator XYZ vector tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorTileData {
    /// Tile column coordinate at the given zoom level.
    pub tile_x: u32,
    /// Tile row coordinate at the given zoom level.
    pub tile_y: u32,
    /// Zoom level this tile belongs to.
    pub zoom_level: u8,
    /// Wire encoding of the byte payload.
    pub tile_encoding: VectorTileEncoding,
    /// Encoded tile bytes (MVT protobuf, optionally gzip-wrapped).
    pub tile_bytes: Vec<u8>,
}

impl VectorTileData {
    /// Builds the record from wire bytes.
    ///
    /// # Parameters
    /// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
    /// - `zoom_level` — the web-mercator zoom level of the tile.
    /// - `wire_bytes` — the payload exactly as delivered by the provider.
    ///
    /// # Returns
    /// A tile record whose encoding is identified from the leading two-byte
    /// gzip magic number, so the record names the encoding it actually
    /// carries.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the payload is empty;
    /// an empty payload is never treated as an empty tile.
    pub fn from_wire_bytes(
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
        wire_bytes: Vec<u8>,
    ) -> EveryMapResult<Self> {
        let tile_encoding = Self::sniff_tile_encoding(&wire_bytes)?;
        Ok(Self {
            tile_x,
            tile_y,
            zoom_level,
            tile_encoding,
            tile_bytes: wire_bytes,
        })
    }

    /// Classifies the wire payload by its leading gzip magic number; an
    /// empty payload is a failure, not a silent empty tile.
    fn sniff_tile_encoding(wire_bytes: &[u8]) -> EveryMapResult<VectorTileEncoding> {
        if wire_bytes.is_empty() {
            return Err(EveryMapError::ValidationError(
                "vector tile payload is empty".to_string(),
            ));
        }
        let carries_gzip_magic = wire_bytes[0] == GZIP_MAGIC_FIRST_BYTE
            && wire_bytes.len() > 1
            && wire_bytes[1] == GZIP_MAGIC_SECOND_BYTE;
        if carries_gzip_magic {
            Ok(VectorTileEncoding::GzippedMapboxVectorTile)
        } else {
            Ok(VectorTileEncoding::MapboxVectorTile)
        }
    }
}

/// Description of one vector-tileset source a [`VectorTileProvider`] serves.
///
/// The source name namespaces every cached response for the tileset, so two
/// providers, or one provider serving two tilesets, never share cache entries
/// even for identical zoom/x/y arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorTileSourceDescriptor {
    /// Stable unique name of the tileset (for example
    /// `"provider-name/tileset-name"`), used as the cache namespace key.
    pub source_name: String,
    /// Human-readable attribution line the provider's usage terms require
    /// wherever the imagery is shown.
    pub attribution_text: String,
    /// Lowest zoom level the provider serves.
    pub minimum_zoom_level: u8,
    /// Highest zoom level the provider serves.
    pub maximum_zoom_level: u8,
    /// Encoding every tile from this source arrives in before any
    /// transport-level decompression.
    pub tile_encoding: VectorTileEncoding,
    /// Style names or families the tileset is designed to be rendered with,
    /// in the provider's preference order. Empty when the source declares
    /// none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub style_hints: Vec<String>,
}

impl VectorTileSourceDescriptor {
    /// Whether the source serves the given zoom level.
    pub fn zoom_span_contains(&self, zoom_level: u8) -> bool {
        (self.minimum_zoom_level..=self.maximum_zoom_level).contains(&zoom_level)
    }
}

/// Vector map imagery served as Mapbox Vector Tile protobuf.
///
/// Each implementation serves one web-mercator XYZ tileset and reports its
/// cache namespace, attribution, zoom span, encoding and style hints through
/// [`Self::source_descriptor`].
#[async_trait]
pub trait VectorTileProvider: Send + Sync {
    /// Describes the served tileset.
    fn source_descriptor(&self) -> VectorTileSourceDescriptor;

    /// Fetches one web-mercator XYZ vector tile in its wire encoding: MVT
    /// protobuf, optionally gzip-wrapped.
    ///
    /// # Parameters
    /// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
    /// - `zoom_level` — the web-mercator zoom level of the tile.
    ///
    /// # Returns
    /// The tile bytes together with the encoding they arrived in.
    ///
    /// # Errors
    /// Returns [`EveryMapError::HttpError`] or
    /// [`EveryMapError::TransportError`] when the request cannot be
    /// completed, [`EveryMapError::ValidationError`] when the payload is
    /// empty or undecodable, and [`EveryMapError::ValidationError`] when the
    /// tile is outside the served zoom span.
    async fn fetch_vector_tile(
        &self,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<VectorTileData>;
}
