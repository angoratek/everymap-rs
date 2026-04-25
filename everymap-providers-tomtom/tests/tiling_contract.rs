use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomTileProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_tiling_contract() {
    let server = MockServer::start().await;

    // Simulate a PNG tile response (8-byte minimal PNG-like header)
    let tile_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    Mock::given(method("GET"))
        .and(path("/map/1/tile/basic/main/10/523/335.png"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(tile_data.clone(), "image/png"))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let tiling = TomTomTileProvider::with_base_url(client, server.uri());

    let opts = TileOptions {
        format: Some("png".to_string()),
        provider_extra: Some(serde_json::json!({
            "layer": "basic",
            "style": "main"
        })),
    };

    let res = tiling.get_tile(10, 523, 335, &opts).await.unwrap();

    assert_eq!(res.data.len(), 8);
    assert_eq!(res.content_type, Some("image/png".to_string()));
}
