//! Contract tests of the TomTom raster-tile provider against a wiremock server.
//!
//! The typed contract serves PNG at the standard resolution; a JPEG or retina
//! request is refused before any request leaves the process. Every request
//! targets the in-process server; no test reaches an external service.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::EveryMapError;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomTileProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The eight-byte PNG signature a rendered tile begins with.
const SAMPLE_PNG_BYTES: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// A provider pointed at the in-process server.
fn provider_for(server: &MockServer) -> TomTomTileProvider {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    TomTomTileProvider::with_base_url(Arc::new(TomTomClient::new(auth)), server.uri())
}

#[tokio::test]
async fn a_standard_png_tile_is_fetched_and_echoed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/map/1/tile/basic/main/10/523/335.png"))
        .and(query_param("tileSize", "256"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(SAMPLE_PNG_BYTES.to_vec(), "image/png"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tile = provider_for(&server)
        .fetch_raster_tile(&RasterTileRequest::new(
            523,
            335,
            10,
            RasterImageFormat::Png,
            RasterTileResolution::Standard,
        ))
        .await
        .expect("the service answers");
    server.verify().await;
    assert_eq!(tile.image_bytes, SAMPLE_PNG_BYTES.to_vec());
    assert_eq!(tile.image_format, RasterImageFormat::Png);
    assert_eq!(tile.tile_resolution, RasterTileResolution::Standard);
}

#[tokio::test]
async fn jpeg_and_retina_requests_are_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let provider = provider_for(&server);

    assert!(matches!(
        provider
            .fetch_raster_tile(&RasterTileRequest::new(
                0,
                0,
                1,
                RasterImageFormat::Jpeg,
                RasterTileResolution::Standard,
            ))
            .await,
        Err(EveryMapError::ValidationError(_))
    ));
    assert!(matches!(
        provider
            .fetch_raster_tile(&RasterTileRequest::new(
                0,
                0,
                1,
                RasterImageFormat::Png,
                RasterTileResolution::Retina,
            ))
            .await,
        Err(EveryMapError::ValidationError(_))
    ));
}

#[tokio::test]
async fn a_zoom_or_index_past_the_served_range_is_refused() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let provider = provider_for(&server);

    assert!(matches!(
        provider
            .fetch_raster_tile(&RasterTileRequest::new(
                0,
                0,
                23,
                RasterImageFormat::Png,
                RasterTileResolution::Standard,
            ))
            .await,
        Err(EveryMapError::ValidationError(_))
    ));
}
