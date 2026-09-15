use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::imaging::HereMapImageProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_imaging_contract() {
    let server = MockServer::start().await;

    let image_data = b"fake_png_image_data";

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/png")
                .set_body_bytes(image_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = ImageOptions::default();

    let response = provider
        .get_image(&center, 10, (512, 512), &options)
        .await
        .unwrap();

    assert_eq!(response.data, image_data.to_vec());
    assert_eq!(response.content_type.as_deref(), Some("image/png"));
}

#[tokio::test]
async fn test_imaging_with_options() {
    let server = MockServer::start().await;

    let image_data = b"fake_jpg_image_data";

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(image_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = ImageOptions {
        format: Some("jpg".to_string()),
        provider_extra: Some(serde_json::json!({
            "style": "default"
        })),
        ..Default::default()
    };

    let response = provider
        .get_image(&center, 12, (256, 256), &options)
        .await
        .unwrap();

    assert_eq!(response.data, image_data.to_vec());
    assert_eq!(response.content_type.as_deref(), Some("image/jpeg"));
}

#[tokio::test]
async fn test_imaging_width_height_override() {
    let server = MockServer::start().await;

    let image_data = b"fake_png_image_data";

    Mock::given(method("GET"))
        .and(path("/base/mc/center:52.52,13.405;zoom=10/640x480/png"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/png")
                .set_body_bytes(image_data.to_vec()),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereMapImageProvider::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = ImageOptions {
        width: Some(640),
        height: Some(480),
        ..Default::default()
    };

    let response = provider
        .get_image(&center, 10, (800, 600), &options)
        .await
        .unwrap();

    assert_eq!(response.data, image_data.to_vec());
    assert_eq!(response.content_type.as_deref(), Some("image/png"));
}
