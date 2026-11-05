//! Contract tests of the HERE vector-tile provider against a wiremock server.
//!
//! HERE's only output segment is `omv`, whose bytes are Mapbox Vector Tile
//! protobuf; the contract requests it for the base layer and echoes the tile.
//! Every request targets the in-process server; no test reaches an external
//! service.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{
    VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::EveryMapError;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::tiling::{
    HereTileProvider, HERE_VECTOR_ATTRIBUTION, HERE_VECTOR_MAXIMUM_ZOOM_LEVEL,
};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A minimal MVT payload: the protobuf bytes stand in for a decoded tile.
const SAMPLE_TILE_BYTES: &[u8] = b"mock_mvt_tile_bytes";

/// A provider pointed at the in-process server.
fn provider_for(server: &MockServer) -> HereTileProvider {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    HereTileProvider::with_base_url(Arc::new(HereClient::new(auth)), server.uri())
}

#[tokio::test]
async fn the_typed_contract_requests_the_omv_output_of_the_base_layer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vectortiles/base/mc/12/2047/1361/omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(SAMPLE_TILE_BYTES.to_vec()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tile = provider_for(&server)
        .fetch_vector_tile(2047, 1361, 12)
        .await
        .expect("the service answers");
    server.verify().await;
    assert_eq!(tile.tile_bytes, SAMPLE_TILE_BYTES.to_vec());
    assert_eq!(tile.tile_encoding, VectorTileEncoding::MapboxVectorTile);
    assert_eq!(
        (tile.tile_x, tile.tile_y, tile.zoom_level),
        (2047, 1361, 12)
    );
}

#[tokio::test]
async fn a_named_layer_is_reachable_through_the_extension() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vectortiles/mapbox/mc/10/511/340/omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(SAMPLE_TILE_BYTES.to_vec()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tile = provider_for(&server)
        .fetch_vector_tile_in_layer("mapbox", 511, 340, 10)
        .await
        .expect("the layer answers");
    server.verify().await;
    assert_eq!(tile.tile_bytes, SAMPLE_TILE_BYTES.to_vec());
}

#[test]
fn the_source_descriptor_names_the_here_tileset() {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let provider = HereTileProvider::new(Arc::new(HereClient::new(auth)));
    assert_eq!(
        provider.source_descriptor(),
        VectorTileSourceDescriptor {
            source_name: "here/vector".to_string(),
            attribution_text: HERE_VECTOR_ATTRIBUTION.to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: HERE_VECTOR_MAXIMUM_ZOOM_LEVEL,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["omv".to_string(), "base".to_string()],
        }
    );
    assert_eq!(HERE_VECTOR_MAXIMUM_ZOOM_LEVEL, 17);
}

#[tokio::test]
async fn a_zoom_or_index_past_the_served_range_is_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let provider = provider_for(&server);

    assert!(matches!(
        provider
            .fetch_vector_tile(0, 0, HERE_VECTOR_MAXIMUM_ZOOM_LEVEL + 1)
            .await,
        Err(EveryMapError::ValidationError(_))
    ));
    assert!(matches!(
        provider.fetch_vector_tile(2, 0, 1).await,
        Err(EveryMapError::ValidationError(_))
    ));
}
