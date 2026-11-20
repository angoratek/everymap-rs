//! Contract tests for the MapQuest Static Map API.

use everymap_core::domains::imaging::{ImageOptions, MapImageProvider};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::{
    MapQuestClient, MapQuestMapImageProvider, MapQuestProviderConfiguration,
};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_provider(server: &MockServer) -> MapQuestMapImageProvider {
    let client = Arc::new(MapQuestClient::new(MapQuestProviderConfiguration {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        ..Default::default()
    }));
    MapQuestMapImageProvider::new(client)
}

fn berlin() -> Coordinate {
    Coordinate::new(52.5163, 13.3777).expect("a valid coordinate")
}

#[tokio::test]
async fn returns_the_image_bytes_and_media_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/staticmap/v5/map"))
        .and(query_param("center", "52.5163,13.3777"))
        .and(query_param("zoom", "14"))
        .and(query_param("size", "600,400"))
        .and(query_param("format", "png"))
        .and(query_param("key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(vec![0x89, b'P', b'N', b'G'], "image/png"),
        )
        .mount(&server)
        .await;

    let image = test_provider(&server)
        .get_image(&berlin(), 14, (600, 400), &ImageOptions::default())
        .await
        .expect("the image is served");

    assert_eq!(image.data, vec![0x89, b'P', b'N', b'G']);
    assert_eq!(image.content_type.as_deref(), Some("image/png"));
}

#[tokio::test]
async fn width_and_height_override_the_size_argument() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/staticmap/v5/map"))
        .and(query_param("size", "1024,768"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(vec![1], "image/png"))
        .mount(&server)
        .await;

    let options = ImageOptions {
        width: Some(1024),
        height: Some(768),
        ..Default::default()
    };
    test_provider(&server)
        .get_image(&berlin(), 14, (600, 400), &options)
        .await
        .expect("the image is served");
}

#[tokio::test]
async fn an_empty_size_is_a_validation_error() {
    let server = MockServer::start().await;
    let error = test_provider(&server)
        .get_image(&berlin(), 14, (0, 400), &ImageOptions::default())
        .await
        .expect_err("an empty size is refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn an_undocumented_format_is_a_validation_error() {
    let server = MockServer::start().await;
    let options = ImageOptions {
        format: Some("webp".to_string()),
        ..Default::default()
    };
    let error = test_provider(&server)
        .get_image(&berlin(), 14, (600, 400), &options)
        .await
        .expect_err("an undocumented format is refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_json_error_envelope_is_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/staticmap/v5/map"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(
                serde_json::json!({
                    "info": {
                        "statuscode": 403,
                        "messages": ["This key is not authorized for this service"]
                    }
                })
                .to_string(),
                "application/json",
            ),
        )
        .mount(&server)
        .await;

    let error = test_provider(&server)
        .get_image(&berlin(), 14, (600, 400), &ImageOptions::default())
        .await
        .expect_err("the service refusal is reported");
    assert!(matches!(error, EveryMapError::ProviderError { .. }));
}
