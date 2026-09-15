use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxMapImageProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_imaging_contract() {
    let server = MockServer::start().await;

    // Simulate a PNG image response
    let image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    Mock::given(method("GET"))
        .and(path(
            "/styles/v1/mapbox/streets-v12/static/13.405,52.52,10/512x512@2x",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(image_data.clone(), "image/png"))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let imaging = MapBoxMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = ImageOptions {
        format: Some("png".to_string()),
        language: None,
        ..Default::default()
    };

    let response = imaging
        .get_image(&center, 10, (512, 512), &options)
        .await
        .unwrap();

    assert_eq!(response.data.len(), 8);
    assert_eq!(response.content_type, Some("image/png".to_string()));
}

#[tokio::test]
async fn test_imaging_width_height_override() {
    let server = MockServer::start().await;

    let image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    Mock::given(method("GET"))
        .and(path(
            "/styles/v1/mapbox/streets-v12/static/13.405,52.52,10/640x480@2x",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(image_data.clone(), "image/png"))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let imaging = MapBoxMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = ImageOptions {
        width: Some(640),
        height: Some(480),
        ..Default::default()
    };

    let response = imaging
        .get_image(&center, 10, (800, 600), &options)
        .await
        .unwrap();

    assert_eq!(response.data.len(), 8);
    assert_eq!(response.content_type, Some("image/png".to_string()));
}
