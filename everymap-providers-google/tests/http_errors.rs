use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::error::EveryMapError;
use everymap_providers_google::GoogleGeocoder;
use everymap_providers_google::client::GoogleClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_geocode_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "code": 401,
                "message": "API key is invalid"
            }
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("invalid-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 401),
        EveryMapError::AuthError { .. } => {}
        other => panic!("Expected HttpError(401) or AuthError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_geocode_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": {
                "code": 403,
                "message": "Access denied"
            }
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

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
                    "error": {
                        "code": 429,
                        "message": "Too Many Requests"
                    }
                }))
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

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
            "error": {
                "code": 500,
                "message": "Internal Server Error"
            }
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let geocoder = GoogleGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 500),
        other => panic!("Expected HttpError with status 500, got {:?}", other),
    }
}