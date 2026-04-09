use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::tiling::{TileRequest, TileProvider};
use everymap_providers_here::domain::tiling::{HereTileProvider, HereTileOptions, TileLayer};
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_tiling_contract() {
    let server = MockServer::start().await;

    // Vector tiles are binary protobuf data
    let tile_data = b"mock_protobuf_tile_data";

    Mock::given(method("GET"))
        .and(path("/vectortiles/mapbox/10/511/340.omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let req = TileRequest {
        z: 10,
        x: 511,
        y: 340,
        options: HereTileOptions {
            layer: TileLayer::Mapbox,
            ..Default::default()
        },
    };

    let res = provider.get_tile(req).await.unwrap();

    assert_eq!(res.data, tile_data.to_vec());
    assert_eq!(res.content_type.as_deref(), Some("application/x-protobuf"));
}

#[tokio::test]
async fn test_tiling_with_different_layer() {
    let server = MockServer::start().await;

    let tile_data = b"mock_base_tile_data";

    Mock::given(method("GET"))
        .and(path("/vectortiles/base/12/2047/1361.omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let req = TileRequest {
        z: 12,
        x: 2047,
        y: 1361,
        options: HereTileOptions {
            layer: TileLayer::Base,
            ..Default::default()
        },
    };

    let res = provider.get_tile(req).await.unwrap();

    assert_eq!(res.data, tile_data.to_vec());
}

#[tokio::test]
async fn test_tiling_with_optional_params() {
    let server = MockServer::start().await;

    let tile_data = b"mock_tile_with_params";

    Mock::given(method("GET"))
        .and(path("/vectortiles/mapbox/10/511/340.omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let req = TileRequest {
        z: 10,
        x: 511,
        y: 340,
        options: HereTileOptions {
            layer: TileLayer::Mapbox,
            political_view: Some("CHN".to_string()),
            ..Default::default()
        },
    };

    let res = provider.get_tile(req).await.unwrap();
    assert_eq!(res.data, tile_data.to_vec());
}