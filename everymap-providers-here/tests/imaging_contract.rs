use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::imaging::HereMapImageProvider;
use std::sync::Arc;
use wiremock::matchers::method;
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
    let opts = ImageOptions::default();

    let res = provider
        .get_image(&center, 10, (512, 512), &opts)
        .await
        .unwrap();

    assert_eq!(res.data, image_data.to_vec());
    assert_eq!(res.content_type.as_deref(), Some("image/png"));
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
    let opts = ImageOptions {
        format: Some("jpg".to_string()),
        provider_extra: Some(serde_json::json!({
            "style": "default"
        })),
        ..Default::default()
    };

    let res = provider
        .get_image(&center, 12, (256, 256), &opts)
        .await
        .unwrap();

    assert_eq!(res.data, image_data.to_vec());
    assert_eq!(res.content_type.as_deref(), Some("image/jpeg"));
}
