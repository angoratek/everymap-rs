//! Contract tests for the typed raster and vector tile surfaces.
//!
//! Fixture bytes are committed under `tests/fixtures/` and served by an
//! in-process HTTP transport; no test reaches the network.

use async_trait::async_trait;
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileData, RasterTileProvider, RasterTileRequest, RasterTileResolution,
    VectorTileData, VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
use std::sync::{Arc, Mutex};

/// 1x1 RGBA PNG committed as the raster fixture.
const RASTER_TILE_BYTES: &[u8] = include_bytes!("fixtures/raster_tile.png");
/// Five-byte MVT protobuf carrying one named layer.
const VECTOR_TILE_BYTES: &[u8] = include_bytes!("fixtures/vector_tile.mvt");
/// The same MVT payload wrapped in gzip.
const GZIPPED_VECTOR_TILE_BYTES: &[u8] = include_bytes!("fixtures/vector_tile.mvt.gz");

/// Serves the committed fixtures by URL and records every requested URL.
#[derive(Default)]
struct FixtureTransport {
    requested_urls: Mutex<Vec<String>>,
}

#[async_trait]
impl HttpTransport for FixtureTransport {
    async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
        let url = request.url_with_query();
        self.requested_urls.lock().unwrap().push(url.clone());

        let (status, content_type, body) = if url.contains("/raster/") {
            (200, "image/png", RASTER_TILE_BYTES.to_vec())
        } else if url.contains("/vector/gzip/") {
            (
                200,
                "application/x-protobuf",
                GZIPPED_VECTOR_TILE_BYTES.to_vec(),
            )
        } else if url.contains("/vector/plain/") {
            (200, "application/x-protobuf", VECTOR_TILE_BYTES.to_vec())
        } else if url.contains("/vector/empty/") {
            (200, "application/x-protobuf", Vec::new())
        } else {
            (404, "text/plain", b"not found".to_vec())
        };

        Ok(HttpResponse::new(
            status,
            HttpHeaders::new(vec![("content-type".to_string(), content_type.to_string())]),
            body,
        ))
    }
}

/// Raster provider that requests the fixture and echoes the request fields.
struct FixtureRasterTileProvider {
    transport: Arc<dyn HttpTransport>,
}

#[async_trait]
impl RasterTileProvider for FixtureRasterTileProvider {
    async fn fetch_raster_tile(
        &self,
        request: &RasterTileRequest,
    ) -> EveryMapResult<RasterTileData> {
        let url = format!(
            "https://tiles.example/raster/{}/{}/{}.{}?tile_resolution={}",
            request.zoom_level,
            request.tile_x,
            request.tile_y,
            request.image_format.file_extension(),
            request.tile_resolution.query_value()
        );
        let response = self.transport.send(HttpRequest::get(url)).await?;
        if !response.is_success() {
            return Err(EveryMapError::http(
                response.status(),
                "raster tile fetch failed",
            ));
        }
        Ok(RasterTileData::for_request(request, response.into_body()))
    }
}

/// Vector provider serving one fixture payload kind behind a descriptor.
struct FixtureVectorTileProvider {
    transport: Arc<dyn HttpTransport>,
    descriptor: VectorTileSourceDescriptor,
    payload_kind: &'static str,
}

#[async_trait]
impl VectorTileProvider for FixtureVectorTileProvider {
    fn source_descriptor(&self) -> VectorTileSourceDescriptor {
        self.descriptor.clone()
    }

    async fn fetch_vector_tile(
        &self,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<VectorTileData> {
        if !self.descriptor.zoom_span_contains(zoom_level) {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {zoom_level} is outside the {}=..={} span of {}",
                self.descriptor.minimum_zoom_level,
                self.descriptor.maximum_zoom_level,
                self.descriptor.source_name
            )));
        }
        let url = format!(
            "https://tiles.example/vector/{}/{}/{}/{}.mvt",
            self.payload_kind, zoom_level, tile_x, tile_y
        );
        let response = self.transport.send(HttpRequest::get(url)).await?;
        if !response.is_success() {
            return Err(EveryMapError::http(
                response.status(),
                "vector tile fetch failed",
            ));
        }
        VectorTileData::from_wire_bytes(tile_x, tile_y, zoom_level, response.into_body())
    }
}

fn descriptor(tile_encoding: VectorTileEncoding) -> VectorTileSourceDescriptor {
    VectorTileSourceDescriptor {
        source_name: "fixture/streets".to_string(),
        attribution_text: "Map data © OpenStreetMap contributors".to_string(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 14,
        tile_encoding,
        style_hints: vec!["openmaptiles".to_string(), "bright".to_string()],
    }
}

#[tokio::test]
async fn raster_tile_echoes_format_and_resolution_from_the_request() {
    let transport = Arc::new(FixtureTransport::default());
    let provider = FixtureRasterTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
    };
    let request = RasterTileRequest::new(
        523,
        335,
        10,
        RasterImageFormat::Png,
        RasterTileResolution::Retina,
    );

    let tile = provider.fetch_raster_tile(&request).await.unwrap();

    let requested_urls = transport.requested_urls.lock().unwrap();
    assert_eq!(requested_urls.len(), 1);
    assert!(
        requested_urls[0].contains("/raster/10/523/335.png"),
        "the format selects the file extension: {}",
        requested_urls[0]
    );
    assert!(requested_urls[0].contains("tile_resolution=retina"));

    assert_eq!(tile.tile_x, 523);
    assert_eq!(tile.tile_y, 335);
    assert_eq!(tile.zoom_level, 10);
    assert_eq!(tile.image_format, RasterImageFormat::Png);
    assert_eq!(tile.tile_resolution, RasterTileResolution::Retina);
    assert_eq!(tile.image_bytes, RASTER_TILE_BYTES);
}

#[tokio::test]
async fn vector_source_descriptor_is_reported_verbatim() {
    let transport = Arc::new(FixtureTransport::default());
    let source_descriptor = descriptor(VectorTileEncoding::GzippedMapboxVectorTile);
    let provider = FixtureVectorTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
        descriptor: source_descriptor.clone(),
        payload_kind: "gzip",
    };

    assert_eq!(provider.source_descriptor(), source_descriptor);
    assert_eq!(
        provider.source_descriptor().style_hints,
        vec!["openmaptiles", "bright"]
    );
    assert!(provider.source_descriptor().zoom_span_contains(14));
    assert!(!provider.source_descriptor().zoom_span_contains(15));
}

#[tokio::test]
async fn plain_vector_tile_payload_is_classified_from_its_wire_bytes() {
    let transport = Arc::new(FixtureTransport::default());
    let provider = FixtureVectorTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
        descriptor: descriptor(VectorTileEncoding::MapboxVectorTile),
        payload_kind: "plain",
    };

    let tile = provider.fetch_vector_tile(4, 5, 6).await.unwrap();

    assert_eq!(tile.tile_x, 4);
    assert_eq!(tile.tile_y, 5);
    assert_eq!(tile.zoom_level, 6);
    assert_eq!(tile.tile_encoding, VectorTileEncoding::MapboxVectorTile);
    assert_eq!(tile.tile_bytes, VECTOR_TILE_BYTES);
}

#[tokio::test]
async fn gzipped_vector_tile_payload_is_classified_from_its_magic_bytes() {
    let transport = Arc::new(FixtureTransport::default());
    let provider = FixtureVectorTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
        descriptor: descriptor(VectorTileEncoding::GzippedMapboxVectorTile),
        payload_kind: "gzip",
    };

    let tile = provider.fetch_vector_tile(1, 2, 3).await.unwrap();

    assert_eq!(
        tile.tile_encoding,
        VectorTileEncoding::GzippedMapboxVectorTile
    );
    assert_eq!(tile.tile_bytes, GZIPPED_VECTOR_TILE_BYTES);
}

#[tokio::test]
async fn vector_tile_outside_the_declared_zoom_span_is_rejected_before_the_network() {
    let transport = Arc::new(FixtureTransport::default());
    let provider = FixtureVectorTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
        descriptor: descriptor(VectorTileEncoding::MapboxVectorTile),
        payload_kind: "plain",
    };

    let error = provider.fetch_vector_tile(0, 0, 15).await.unwrap_err();

    assert!(matches!(error, EveryMapError::ValidationError(_)));
    assert!(transport.requested_urls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn empty_vector_tile_payload_is_rejected() {
    let transport = Arc::new(FixtureTransport::default());
    let provider = FixtureVectorTileProvider {
        transport: Arc::clone(&transport) as Arc<dyn HttpTransport>,
        descriptor: descriptor(VectorTileEncoding::MapboxVectorTile),
        payload_kind: "empty",
    };

    let error = provider.fetch_vector_tile(0, 0, 3).await.unwrap_err();

    assert!(matches!(error, EveryMapError::ValidationError(_)));
}
