use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxTileProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_tiling_contract() {
    let server = MockServer::start().await;

    // Simulate a MVT tile response
    let tile_data = vec![0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F];

    Mock::given(method("GET"))
        .and(path("/v4/mapbox.mapbox-streets-v8/10/523/335.mvt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(tile_data.clone(), "application/vnd.mapbox-vector-tile"),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let tiling = MapBoxTileProvider::with_base_url(client, server.uri());

    let opts = TileOptions {
        format: Some("mvt".to_string()),
        provider_extra: Some(serde_json::json!({
            "tileset_id": "mapbox.mapbox-streets-v8"
        })),
    };

    let res = tiling.get_tile(10, 523, 335, &opts).await.unwrap();

    assert_eq!(res.data.len(), 6);
    assert_eq!(
        res.content_type,
        Some("application/vnd.mapbox-vector-tile".to_string())
    );
}
