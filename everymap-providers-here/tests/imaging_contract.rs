use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use everymap_core::types::Coordinate;
use everymap_core::domains::imaging::{ImageRequest, MapImageProvider};
use everymap_providers_here::domain::imaging::{HereMapImageProvider, HereImageOptions, ImageFormat};
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereMapImageProvider::with_base_url(client, server.uri());

    let req = ImageRequest {
        center: Coordinate::new(52.52, 13.405).unwrap(),
        zoom: 10,
        size: (512, 512),
        options: HereImageOptions {
            format: ImageFormat::Png,
            ..Default::default()
        },
    };

    let res = provider.get_image(req).await.unwrap();

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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereMapImageProvider::with_base_url(client, server.uri());

    let req = ImageRequest {
        center: Coordinate::new(52.52, 13.405).unwrap(),
        zoom: 12,
        size: (256, 256),
        options: HereImageOptions {
            format: ImageFormat::Jpg,
            style: Some("default".to_string()),
            ..Default::default()
        },
    };

    let res = provider.get_image(req).await.unwrap();

    assert_eq!(res.data, image_data.to_vec());
    assert_eq!(res.content_type.as_deref(), Some("image/jpeg"));
}