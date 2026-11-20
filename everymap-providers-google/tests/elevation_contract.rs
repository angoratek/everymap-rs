//! Contract tests for the Google Maps Elevation API.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::elevation::ElevationProvider;
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleElevationProvider;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_provider(server: &MockServer) -> GoogleElevationProvider {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    GoogleElevationProvider::with_base_url(client, server.uri())
}

#[tokio::test]
async fn samples_the_requested_points_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/maps/api/elevation/json"))
        .and(query_param("locations", "52.5163,13.3777|48.8584,2.2945"))
        .and(query_param("key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {
                    "elevation": 34.0,
                    "location": { "lat": 52.5163, "lng": 13.3777 },
                    "resolution": 19.0
                },
                {
                    "elevation": 35.0,
                    "location": { "lat": 48.8584, "lng": 2.2945 },
                    "resolution": 19.0
                }
            ],
            "status": "OK"
        })))
        .mount(&server)
        .await;

    let points = [
        Coordinate::new(52.5163, 13.3777).expect("a valid coordinate"),
        Coordinate::new(48.8584, 2.2945).expect("a valid coordinate"),
    ];
    let samples = test_provider(&server)
        .elevation_points(&points)
        .await
        .expect("the contract call succeeds");

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].elevation_meters, 34.0);
    assert_eq!(samples[0].coordinate.lat, 52.5163);
    assert_eq!(samples[1].elevation_meters, 35.0);
    assert_eq!(samples[1].coordinate.lng, 2.2945);
}

#[tokio::test]
async fn an_empty_point_list_is_a_validation_error() {
    let server = MockServer::start().await;
    let error = test_provider(&server)
        .elevation_points(&[])
        .await
        .expect_err("an empty list is refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_denied_request_maps_to_an_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/maps/api/elevation/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [],
            "status": "REQUEST_DENIED",
            "error_message": "The provided API key is invalid."
        })))
        .mount(&server)
        .await;

    let error = test_provider(&server)
        .elevation_points(&[Coordinate::new(52.5, 13.4).expect("a valid coordinate")])
        .await
        .expect_err("the denied request fails");
    assert!(matches!(error, EveryMapError::AuthError { .. }));
}
