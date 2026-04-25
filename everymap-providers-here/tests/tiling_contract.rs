use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::tiling::HereTileProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_tiling_contract() {
    let server = MockServer::start().await;

    // Vector tiles are binary protobuf data
    let tile_data = b"mock_protobuf_tile_data";

    Mock::given(method("GET"))
        .and(path("/vectortiles/mapbox/mc/10/511/340/omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let opts = TileOptions {
        provider_extra: Some(serde_json::json!({
            "layer": "mapbox"
        })),
        ..Default::default()
    };

    let res = provider.get_tile(10, 511, 340, &opts).await.unwrap();

    assert_eq!(res.data, tile_data.to_vec());
    assert_eq!(res.content_type.as_deref(), Some("application/x-protobuf"));
}

#[tokio::test]
async fn test_tiling_with_different_layer() {
    let server = MockServer::start().await;

    let tile_data = b"mock_base_tile_data";

    Mock::given(method("GET"))
        .and(path("/vectortiles/base/mc/12/2047/1361/omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let opts = TileOptions {
        provider_extra: Some(serde_json::json!({
            "layer": "base"
        })),
        ..Default::default()
    };

    let res = provider.get_tile(12, 2047, 1361, &opts).await.unwrap();

    assert_eq!(res.data, tile_data.to_vec());
}

#[tokio::test]
async fn test_tiling_with_optional_params() {
    let server = MockServer::start().await;

    let tile_data = b"mock_tile_with_params";

    Mock::given(method("GET"))
        .and(path("/vectortiles/mapbox/mc/10/511/340/omv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(tile_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereTileProvider::with_base_url(client, server.uri());

    let opts = TileOptions {
        provider_extra: Some(serde_json::json!({
            "layer": "mapbox",
            "political_view": "CHN"
        })),
        ..Default::default()
    };

    let res = provider.get_tile(10, 511, 340, &opts).await.unwrap();
    assert_eq!(res.data, tile_data.to_vec());
}
