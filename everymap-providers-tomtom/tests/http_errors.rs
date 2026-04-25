use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{GeocodeOptions, Geocoder};
use everymap_core::error::EveryMapError;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomGeocoder;
use std::sync::Arc;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_geocode_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Unauthorized",
            "error_description": "API key is invalid"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "invalid-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 401),
        other => panic!("Expected HttpError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_geocode_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": "Forbidden",
            "error_description": "Access denied"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 403),
        other => panic!("Expected HttpError with status 403, got {:?}", other),
    }
}

#[tokio::test]
async fn test_geocode_rate_limited() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_json(serde_json::json!({
                    "error": "Too Many Requests"
                })),
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 429),
        EveryMapError::RateLimited { .. } => {}
        other => panic!("Expected HttpError(429) or RateLimited, got {:?}", other),
    }
}

#[tokio::test]
async fn test_geocode_server_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": "Internal Server Error"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let geocoder = TomTomGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 500),
        other => panic!("Expected HttpError with status 500, got {:?}", other),
    }
}
