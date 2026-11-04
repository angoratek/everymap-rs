use crate::error::{EveryMapError, EveryMapResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for map tile retrieval.
///
/// This is the original untyped tile surface. New code uses
/// [`RasterTileProvider`] or [`VectorTileProvider`], which carry the tile
/// format, resolution and source metadata as typed values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileOptions {
    /// Response format (e.g., "protobuf", "json")
    pub format: Option<String>,
    /// Provider-specific options (HERE: layer, political_view; Mapbox: style_id, access_token)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Simplified tile response from the core trait.
#[derive(Debug, Clone)]
pub struct TileResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

impl TileResponse {
    /// Create a new tile response from raw bytes.
    pub fn new(data: Vec<u8>, content_type: Option<String>) -> Self {
        Self { data, content_type }
    }
}

/// Original untyped tile retrieval.
///
/// Kept for source compatibility. Prefer [`RasterTileProvider`] for encoded
/// imagery and [`VectorTileProvider`] for vector tilesets.
#[async_trait]
pub trait TileProvider: Send + Sync {
    async fn get_tile(
        &self,
        z: u32,
        x: u32,
        y: u32,
        options: &TileOptions,
    ) -> EveryMapResult<TileResponse>;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_options_default() {
        let options = TileOptions::default();
        assert!(options.format.is_none());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_tile_options_with_fields() {
        let options = TileOptions {
            format: Some("protobuf".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "base"})),
        };
        assert_eq!(options.format, Some("protobuf".to_string()));
    }

    #[test]
    fn test_tile_response_construction() {
        let response = TileResponse::new(vec![1, 2, 3], Some("application/x-protobuf".to_string()));
        assert_eq!(response.data.len(), 3);
        assert_eq!(
            response.content_type,
            Some("application/x-protobuf".to_string())
        );
    }

    // --- TileOptions serde roundtrip ---

    #[test]
    fn test_tile_options_serde_roundtrip() {
        let options = TileOptions {
            format: Some("json".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "base", "political_view": "ARG"})),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: TileOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, Some("json".to_string()));
        assert!(back.provider_extra.is_some());
    }

    // --- TileResponse edge cases ---

    #[test]
    fn test_tile_response_empty_data() {
        let response = TileResponse::new(vec![], None);
        assert!(response.data.is_empty());
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_tile_response_no_content_type() {
        let response = TileResponse::new(vec![0x1A, 0x2B, 0x3C, 0x4D], None);
        assert_eq!(response.data.len(), 4);
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_tile_response_json_content_type() {
        let response = TileResponse::new(
            b"{\"tile\":1}".to_vec(),
            Some("application/json".to_string()),
        );
        assert_eq!(response.content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_tile_response_large_data() {
        let data: Vec<u8> = (0..8192).map(|i| (i % 256) as u8).collect();
        let response = TileResponse::new(data, Some("application/x-protobuf".to_string()));
        assert_eq!(response.data.len(), 8192);
    }

    #[test]
    fn test_tile_response_clone() {
        let response = TileResponse::new(vec![1, 2, 3], Some("test/type".to_string()));
        let cloned = response.clone();
        assert_eq!(cloned.data, response.data);
        assert_eq!(cloned.content_type, response.content_type);
    }

    #[test]
    fn test_tile_response_debug() {
        let response = TileResponse::new(vec![1], Some("application/json".to_string()));
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("TileResponse"));
    }

    // --- Typed raster contract ---

    #[test]
    fn test_raster_image_format_serde_and_extension() {
        assert_eq!(
            serde_json::to_string(&RasterImageFormat::Png).unwrap(),
            "\"png\""
        );
        let jpeg: RasterImageFormat = serde_json::from_str("\"jpeg\"").unwrap();
        assert_eq!(jpeg, RasterImageFormat::Jpeg);
        assert_eq!(RasterImageFormat::Png.file_extension(), "png");
        assert_eq!(RasterImageFormat::Jpeg.file_extension(), "jpg");
    }

    #[test]
    fn test_raster_resolution_serde_and_default() {
        assert_eq!(
            RasterTileResolution::default(),
            RasterTileResolution::Standard
        );
        assert_eq!(RasterTileResolution::Retina.query_value(), "retina");
        let back: RasterTileResolution = serde_json::from_str("\"retina\"").unwrap();
        assert_eq!(back, RasterTileResolution::Retina);
    }

    #[test]
    fn test_raster_tile_data_echoes_request() {
        let request = RasterTileRequest::new(
            523,
            335,
            10,
            RasterImageFormat::Jpeg,
            RasterTileResolution::Retina,
        );
        let data = RasterTileData::for_request(&request, vec![0xFF, 0xD8]);
        assert_eq!(data.tile_x, request.tile_x);
        assert_eq!(data.tile_y, request.tile_y);
        assert_eq!(data.zoom_level, request.zoom_level);
        assert_eq!(data.image_format, RasterImageFormat::Jpeg);
        assert_eq!(data.tile_resolution, RasterTileResolution::Retina);
        assert_eq!(data.image_bytes, vec![0xFF, 0xD8]);
    }

    #[test]
    fn test_raster_tile_data_serde_roundtrip() {
        let data = RasterTileData {
            tile_x: 1,
            tile_y: 2,
            zoom_level: 3,
            image_format: RasterImageFormat::Png,
            tile_resolution: RasterTileResolution::Standard,
            image_bytes: vec![0x89, 0x50],
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: RasterTileData = serde_json::from_str(&json).unwrap();
        assert_eq!(back, data);
    }

    #[test]
    fn test_raster_tile_data_legacy_json_defaults_resolution() {
        let legacy = serde_json::json!({
            "tile_x": 1,
            "tile_y": 2,
            "zoom_level": 3,
            "image_format": "png",
            "image_bytes": [1, 2, 3]
        });
        let data: RasterTileData = serde_json::from_value(legacy).unwrap();
        assert_eq!(data.tile_resolution, RasterTileResolution::Standard);
    }

    // --- Typed vector contract ---

    /// Protobuf key for MVT's repeated `layers` field (field 3, wire type 2).
    const LAYER_FIELD_PROTOBUF_KEY: u8 = 0x1a;

    #[test]
    fn test_vector_plain_protobuf_bytes_classify() {
        let tile =
            VectorTileData::from_wire_bytes(1, 2, 3, vec![LAYER_FIELD_PROTOBUF_KEY, 0]).unwrap();
        assert_eq!(tile.tile_encoding, VectorTileEncoding::MapboxVectorTile);
        assert_eq!(tile.tile_bytes, vec![LAYER_FIELD_PROTOBUF_KEY, 0]);
    }

    #[test]
    fn test_vector_gzip_magic_bytes_classify() {
        let wire_bytes = [GZIP_MAGIC_FIRST_BYTE, GZIP_MAGIC_SECOND_BYTE, 0x08].to_vec();
        let tile = VectorTileData::from_wire_bytes(4, 5, 6, wire_bytes).unwrap();
        assert_eq!(
            tile.tile_encoding,
            VectorTileEncoding::GzippedMapboxVectorTile
        );
    }

    #[test]
    fn test_vector_empty_payload_is_rejected() {
        assert!(matches!(
            VectorTileData::from_wire_bytes(0, 0, 0, Vec::new()),
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[test]
    fn test_vector_source_descriptor_serde_and_zoom_span() {
        let descriptor = VectorTileSourceDescriptor {
            source_name: "example/streets".to_string(),
            attribution_text: "Map data © OpenStreetMap contributors".to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 14,
            tile_encoding: VectorTileEncoding::GzippedMapboxVectorTile,
            style_hints: vec!["openmaptiles".to_string(), "bright".to_string()],
        };
        assert!(descriptor.zoom_span_contains(0));
        assert!(descriptor.zoom_span_contains(14));
        assert!(!descriptor.zoom_span_contains(15));

        let json = serde_json::to_string(&descriptor).unwrap();
        let back: VectorTileSourceDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(back, descriptor);
        assert_eq!(back.style_hints.len(), 2);
    }

    #[test]
    fn test_vector_source_descriptor_style_hints_default_empty() {
        let legacy = serde_json::json!({
            "source_name": "example/streets",
            "attribution_text": "attribution",
            "minimum_zoom_level": 0,
            "maximum_zoom_level": 14,
            "tile_encoding": "mapbox_vector_tile"
        });
        let descriptor: VectorTileSourceDescriptor = serde_json::from_value(legacy).unwrap();
        assert!(descriptor.style_hints.is_empty());
        assert_eq!(
            descriptor.tile_encoding,
            VectorTileEncoding::MapboxVectorTile
        );
    }
}
