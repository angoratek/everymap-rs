use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleMapImageProvider;
use std::sync::Arc;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_imaging_contract() {
    let server = MockServer::start().await;

    let png_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header

    // The GoogleMapImageProvider uses base_url directly (no path appended),
    // so when we set base_url to server.uri(), requests go to "/"
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(png_bytes)
                .insert_header("content-type", "image/png"),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let opts = ImageOptions::default();

    let res = provider
        .get_image(&center, 14, (800, 600), &opts)
        .await
        .unwrap();

    assert!(!res.data.is_empty());
    assert_eq!(res.content_type, Some("image/png".to_string()));
}

#[tokio::test]
async fn test_imaging_with_format_and_language() {
    let server = MockServer::start().await;

    let jpg_bytes: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(jpg_bytes)
                .insert_header("content-type", "image/jpeg"),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(40.7128, -74.006).unwrap();
    let opts = ImageOptions {
        format: Some("jpg".to_string()),
        language: Some("en".to_string()),
        provider_extra: None,
    };

    let res = provider
        .get_image(&center, 10, (600, 400), &opts)
        .await
        .unwrap();

    assert!(!res.data.is_empty());
    assert_eq!(res.content_type, Some("image/jpeg".to_string()));
}
