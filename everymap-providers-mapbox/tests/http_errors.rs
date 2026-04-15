use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::error::EveryMapError;
use everymap_providers_mapbox::MapBoxGeocoder;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_geocode_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "message": "Not Authorized - Invalid Token"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("invalid-token".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

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
            "message": "Forbidden - Account does not have access"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-token".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

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
                    "message": "Too Many Requests"
                }))
        )
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-token".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

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
            "message": "Internal Server Error"
        })))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-token".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let geocoder = MapBoxGeocoder::with_base_url(client, server.uri());

    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 500),
        other => panic!("Expected HttpError with status 500, got {:?}", other),
    }
}