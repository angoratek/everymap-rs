use everymap_core::domains::tracking::{TripTracker, TripCreateOptions, TripUpdateOptions, TripStatus};
use everymap_core::auth::{AuthProvider, HeaderAuthProvider};
use everymap_providers_radar::{RadarTripTracker, RadarClient};
use everymap_core::types::Coordinate;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::sync::Arc;

async fn setup_tracker_mock() -> (MockServer, RadarTripTracker) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> = Arc::new(HeaderAuthProvider::new("prj_test_pk_123".to_string()));
    let client = Arc::new(RadarClient::new(auth));
    let tracker = RadarTripTracker::with_base_url(
        client,
        format!("{}/v1/trips", server.uri()),
    );
    (server, tracker)
}

#[tokio::test]
async fn test_create_trip_contract() {
    let (server, tracker) = setup_tracker_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/trips"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "trip": {
                "_id": "trip_abc123",
                "externalId": "ext_1",
                "status": "pending",
                "origin": { "latitude": 40.7128, "longitude": -74.006 },
                "destination": { "latitude": 42.3601, "longitude": -71.0589 },
                "mode": "car",
                "eta": "2024-06-01T14:00:00Z",
                "tag": "delivery"
            }
        })))
        .mount(&server)
        .await;

    let opts = TripCreateOptions {
        origin: Some(Coordinate::new(40.7128, -74.006).unwrap()),
        destination: Some(Coordinate::new(42.3601, -71.0589).unwrap()),
        mode: Some("car".to_string()),
        external_id: Some("ext_1".to_string()),
        tag: Some("delivery".to_string()),
        ..Default::default()
    };
    let result = tracker.create_trip(&opts).await.unwrap();
    assert_eq!(result.id, "trip_abc123");
    assert!(matches!(result.status, Some(TripStatus::Pending)));
    assert_eq!(result.mode.as_deref(), Some("car"));
    assert!(result.origin.is_some());
    assert!(result.destination.is_some());
}

#[tokio::test]
async fn test_update_trip_contract() {
    let (server, tracker) = setup_tracker_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/trips/trip_abc123/update"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "trip": {
                "_id": "trip_abc123",
                "status": "started",
                "mode": "car"
            }
        })))
        .mount(&server)
        .await;

    let opts = TripUpdateOptions {
        trip_id: "trip_abc123".to_string(),
        status: Some(TripStatus::Started),
        ..Default::default()
    };
    let result = tracker.update_trip(&opts).await.unwrap();
    assert_eq!(result.id, "trip_abc123");
    assert!(matches!(result.status, Some(TripStatus::Started)));
}

#[tokio::test]
async fn test_get_trip_contract() {
    let (server, tracker) = setup_tracker_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/trips/trip_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "trip": {
                "_id": "trip_abc123",
                "status": "approaching",
                "mode": "car",
                "eta": "2024-06-01T13:30:00Z"
            }
        })))
        .mount(&server)
        .await;

    let result = tracker.get_trip("trip_abc123").await.unwrap();
    assert_eq!(result.id, "trip_abc123");
    assert!(matches!(result.status, Some(TripStatus::Approaching)));
}

#[tokio::test]
async fn test_create_trip_error_response() {
    let (server, tracker) = setup_tracker_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/trips"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 401 },
            "trip": {
                "_id": ""
            }
        })))
        .mount(&server)
        .await;

    let opts = TripCreateOptions::default();
    let result = tracker.create_trip(&opts).await;
    assert!(result.is_err());
}