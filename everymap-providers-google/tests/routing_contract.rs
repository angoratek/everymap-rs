use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{
    AvoidType, DepartureTime, RouteOptions, Router, TransportMode,
};
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleRouter;
use std::sync::Arc;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

const FIELD_MASK: &str = "routes.distanceMeters,routes.duration,routes.travelMode,routes.polyline.encodedPolyline,routes.viewport,routes.legs";

/// Encoded polyline for a simple two-point route.
const ENCODED_POLYLINE: &str = "_m_I??c~_I??~o~@";

/// Exact header matcher. wiremock's `header` matcher splits values on
/// commas, which would break the comma-separated field mask value.
struct ExactHeader(&'static str, &'static str);

impl Match for ExactHeader {
    fn matches(&self, request: &Request) -> bool {
        request
            .headers
            .get(self.0)
            .and_then(|value| value.to_str().ok())
            .map(|value| value == self.1)
            .unwrap_or(false)
    }
}

fn mock_route_response() -> serde_json::Value {
    serde_json::json!({
        "routes": [
            {
                "distanceMeters": 1052000,
                "duration": "36720s",
                "staticDuration": "36000s",
                "travelMode": "DRIVE",
                "polyline": { "encodedPolyline": ENCODED_POLYLINE },
                "viewport": {
                    "high": { "latitude": 52.5170, "longitude": 13.3780 },
                    "low": { "latitude": 48.8566, "longitude": 2.3522 }
                },
                "legs": [
                    {
                        "distanceMeters": 1052000,
                        "duration": "36720s",
                        "startLocation": { "latLng": { "latitude": 52.5163, "longitude": 13.3777 } },
                        "endLocation": { "latLng": { "latitude": 48.8566, "longitude": 2.3522 } },
                        "steps": [
                            {
                                "distanceMeters": 500,
                                "staticDuration": "60s",
                                "startLocation": { "latLng": { "latitude": 52.5163, "longitude": 13.3777 } },
                                "endLocation": { "latLng": { "latitude": 52.5170, "longitude": 13.3780 } },
                                "navigationInstruction": { "instructions": "Head north" },
                                "travelMode": "DRIVE"
                            },
                            {
                                "distanceMeters": 1051500,
                                "staticDuration": "36660s",
                                "startLocation": { "latLng": { "latitude": 52.5170, "longitude": 13.3780 } },
                                "endLocation": { "latLng": { "latitude": 48.8566, "longitude": 2.3522 } },
                                "navigationInstruction": { "instructions": "Continue to Paris" },
                                "travelMode": "DRIVE"
                            }
                        ]
                    }
                ],
                "routeLabels": ["DEFAULT_ROUTE"]
            }
        ]
    })
}

/// Matcher that passes only when the request body does NOT contain the
/// given top-level JSON field.
struct BodyWithoutField(&'static str);

impl Match for BodyWithoutField {
    fn matches(&self, request: &Request) -> bool {
        serde_json::from_slice::<serde_json::Value>(&request.body)
            .map(|body| body.get(self.0).is_none())
            .unwrap_or(false)
    }
}

/// GoogleRouter uses base_url directly as the full URL, so with_base_url
/// replaces the entire URL with the mock server URI (path becomes "/").
fn google_router(server: &MockServer) -> GoogleRouter {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    GoogleRouter::with_base_url(client, server.uri())
}

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    // Assert the Routes API v2 request shape: waypoints as latLng objects
    // and DRIVE travel mode.
    Mock::given(method("POST"))
        .and(ExactHeader("X-Goog-FieldMask", FIELD_MASK))
        .and(body_partial_json(serde_json::json!({
            "origin": { "location": { "latLng": { "latitude": 52.5163, "longitude": 13.3777 } } },
            "destination": { "location": { "latLng": { "latitude": 48.8566, "longitude": 2.3522 } } },
            "travelMode": "DRIVE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);

    let route = &result.routes[0];
    assert_eq!(route.distance, 1052000.0);
    assert_eq!(route.duration, 36720.0);
    assert_eq!(route.transport_mode, Some(TransportMode::Car));
    assert!(!route.geometry.points.is_empty());
    assert_eq!(route.steps.len(), 2);
    assert_eq!(route.steps[0].instruction, Some("Head north".to_string()));
    assert_eq!(route.steps[0].distance, Some(500.0));
    assert_eq!(route.steps[0].duration, Some(60.0));
    assert!(route.steps[0].start_coordinate.is_some());
    assert!(route.steps[0].end_coordinate.is_some());
    assert!(route.bounding_box.is_some());
}

#[tokio::test]
async fn test_routing_with_pedestrian_mode() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(
            serde_json::json!({ "travelMode": "WALK" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Pedestrian),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
    assert_eq!(result.routes[0].distance, 1052000.0);
    assert_eq!(result.routes[0].duration, 36720.0);
}

#[tokio::test]
async fn test_routing_error_response() {
    let server = MockServer::start().await;

    // Routes API v2 reports errors via HTTP status codes, not an in-body
    // status field like the legacy Directions API.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "code": 400,
                "message": "Field mask is missing.",
                "status": "INVALID_ARGUMENT"
            }
        })))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(48.856, 2.352).unwrap();
    let options = RouteOptions::default();

    let result = router.calculate_route(&start, &end, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_routing_zero_results() {
    let server = MockServer::start().await;

    // Zero results arrive as HTTP 200 with an empty routes list.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "routes": []
        })))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(48.856, 2.352).unwrap();
    let options = RouteOptions::default();

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert!(result.routes.is_empty());
}

#[tokio::test]
async fn test_routing_with_avoid_tolls() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "routeModifiers": {
                "avoidTolls": true,
                "avoidFerries": true
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
}

#[tokio::test]
async fn test_routing_with_departure_time() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "departureTime": "2024-06-01T00:00:00Z"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        departure_time: Some(DepartureTime::Timestamp(1717200000)),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
}

#[tokio::test]
async fn test_routing_with_arrival_time() {
    let server = MockServer::start().await;

    // arrivalTime is not supported outside TRANSIT mode: the field must NOT
    // be sent, only a warning is logged, and the request still succeeds.
    Mock::given(method("POST"))
        .and(BodyWithoutField("arrivalTime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        arrival_time: Some(DepartureTime::Timestamp(1717200000)),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
}

#[tokio::test]
async fn test_routing_with_alternatives_and_language() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "computeAlternativeRoutes": true,
            "languageCode": "de-DE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        alternatives: Some(3),
        language: Some("de-DE".to_string()),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
}

#[tokio::test]
async fn test_routing_with_provider_extra_waypoints() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(serde_json::json!({
            "intermediates": [
                { "location": { "latLng": { "latitude": 51.5, "longitude": 7.1 } } },
                { "location": { "latLng": { "latitude": 50.1, "longitude": 8.3 } } }
            ],
            "optimizeWaypoints": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_route_response()))
        .mount(&server)
        .await;

    let router = google_router(&server);
    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        provider_extra: Some(serde_json::json!({
            "waypoints": "51.5,7.1|50.1,8.3",
            "optimize_waypoints": true
        })),
        ..Default::default()
    };

    let result = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
}
