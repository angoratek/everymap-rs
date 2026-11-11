//! Contract tests of the MapBox vector-tile provider against a wiremock server.
//!
//! The typed contract serves the Streets v8 tileset as MVT; the descriptor
//! names the tileset for the cache namespace and the attribution. Every
//! request targets the in-process server; no test reaches an external service.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{
    VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::EveryMapError;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::{
    MapBoxTileProvider, DEFAULT_MAPBOX_TILESET_ID, MAPBOX_VECTOR_ATTRIBUTION,
    MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL,
};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A minimal MVT payload: the protobuf bytes stand in for a decoded tile.
const SAMPLE_TILE_BYTES: &[u8] = b"mock_mvt_tile_bytes";

/// A provider pointed at the in-process server.
fn provider_for(server: &MockServer) -> MapBoxTileProvider {
    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    MapBoxTileProvider::with_base_url(Arc::new(MapBoxClient::new(auth)), server.uri())
}

#[tokio::test]
async fn the_typed_contract_requests_the_streets_tileset_as_mvt() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v4/mapbox.mapbox-streets-v8/10/523/335.mvt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(SAMPLE_TILE_BYTES.to_vec(), "application/x-protobuf"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tile = provider_for(&server)
        .fetch_vector_tile(523, 335, 10)
        .await
        .expect("the service answers");
    server.verify().await;
    assert_eq!(tile.tile_bytes, SAMPLE_TILE_BYTES.to_vec());
    assert_eq!(tile.tile_encoding, VectorTileEncoding::MapboxVectorTile);
}

#[test]
fn the_source_descriptor_names_the_streets_tileset() {
    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let provider = MapBoxTileProvider::new(Arc::new(MapBoxClient::new(auth)));
    assert_eq!(
        provider.source_descriptor(),
        VectorTileSourceDescriptor {
            source_name: format!("mapbox/{DEFAULT_MAPBOX_TILESET_ID}"),
            attribution_text: MAPBOX_VECTOR_ATTRIBUTION.to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["streets".to_string()],
        }
    );
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
            .fetch_vector_tile(0, 0, MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL + 1)
            .await,
        Err(EveryMapError::ValidationError(_))
    ));
    assert!(matches!(
        provider.fetch_vector_tile(2, 0, 1).await,
        Err(EveryMapError::ValidationError(_))
    ));
}

/// A caller-selected tileset replaces the default in both the descriptor and
/// the requested path.
#[tokio::test]
async fn a_selected_tileset_replaces_the_streets_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v4/acme.acme-terrain/10/523/335.mvt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(SAMPLE_TILE_BYTES.to_vec(), "application/x-protobuf"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = provider_for(&server).with_tileset_id("acme.acme-terrain");
    assert_eq!(
        provider.source_descriptor().source_name,
        "mapbox/acme.acme-terrain"
    );
    provider
        .fetch_vector_tile(523, 335, 10)
        .await
        .expect("the selected tileset answers");
    server.verify().await;
}
