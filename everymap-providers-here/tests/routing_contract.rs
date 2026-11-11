//! Contract tests of the HERE router against a wiremock server replaying
//! committed fixtures and hand-authored payloads.
//!
//! The fixture under `tests/fixtures` is a minimal Routing API v8 document
//! with two sections, each carrying a plain-string flexible polyline and its
//! maneuver actions. The tests assert that every section becomes a leg with
//! its own geometry and actions, that actions are requested by default, that
//! the reported capabilities match what the provider honours, and that a
//! malformed shape is a typed failure.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{
    AvoidType, DepartureTime, RouteOptions, RouteStrategy, Router, TransportMode,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::{Coordinate, FlexiblePolyline};
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::routing::HereRouter;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Directory holding the committed routing payloads.
const FIXTURES_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Flexible polyline both fixture sections carry.
const FIXTURE_POLYLINE: &str = "BFoz5xJ67i1B1B7PzIhaxL7Y";

/// Route fields requested when the caller names none.
const DEFAULT_RETURN_FIELDS: &str = "polyline,summary,actions";

/// Reads one committed fixture.
fn fixture(name: &str) -> serde_json::Value {
    let fixture_path = format!("{FIXTURES_DIRECTORY}/{name}");
    serde_json::from_str(
        &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
    )
    .expect("the fixture must be valid JSON")
}

fn router_for(server: &MockServer) -> HereRouter {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    HereRouter::with_base_url(client, server.uri())
}

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "sections": [
                    {
                        "summary": { "length": 1500.0, "duration": 300.0 },
                        "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                    }
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();

    assert_eq!(response.routes[0].distance, 1500.0);
    assert_eq!(response.routes[0].duration, 300.0);
}

#[tokio::test]
async fn test_routing_with_avoid_tolls() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1600.0, "duration": 320.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert!(response.routes[0].distance > 0.0);
}

#[tokio::test]
async fn test_routing_with_alternatives() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "sections": [{
                    "summary": { "length": 1500.0, "duration": 300.0 },
                    "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                }]
            },
            {
                "sections": [{
                    "summary": { "length": 1600.0, "duration": 310.0 },
                    "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                }]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        alternatives: Some(2),
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert_eq!(response.routes.len(), 2);
}

#[tokio::test]
async fn test_routing_with_departure_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1500.0, "duration": 280.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        departure_time: Some(DepartureTime::Timestamp(1715702400)),
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert!(response.routes[0].duration > 0.0);
}

#[tokio::test]
async fn test_routing_with_language() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1500.0, "duration": 300.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        language: Some("de-DE".to_string()),
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    assert!(response.routes[0].distance > 0.0);
}

#[tokio::test]
async fn every_section_becomes_a_leg_with_geometry_and_actions() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .and(query_param("return", DEFAULT_RETURN_FIELDS))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("routing_two_sections.json")),
        )
        .mount(&server)
        .await;

    let router = router_for(&server);
    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let response = router
        .calculate_route(&start, &end, &options)
        .await
        .unwrap();
    let route = &response.routes[0];

    assert_eq!(route.legs.len(), 2, "one leg per response section");
    assert_eq!(route.distance, 2000.0);
    assert_eq!(route.duration, 400.0);

    let expected_shape =
        FlexiblePolyline::decode(FIXTURE_POLYLINE).expect("the fixture polyline must decode");
    assert_eq!(route.legs[0].distance, 1200.0);
    assert_eq!(route.legs[0].duration, 240.0);
    assert_eq!(route.legs[0].shape, expected_shape);
    assert_eq!(route.legs[0].steps.len(), 2);
    assert_eq!(
        route.legs[0].steps[0].instruction.as_deref(),
        Some("Head north on A1")
    );
    assert_eq!(
        route.legs[0].steps[1].instruction.as_deref(),
        Some("Turn right onto B2")
    );

    assert_eq!(route.legs[1].distance, 800.0);
    assert_eq!(route.legs[1].duration, 160.0);
    assert_eq!(route.legs[1].shape, expected_shape);
    assert_eq!(route.legs[1].steps.len(), 1);
    assert_eq!(
        route.legs[1].steps[0].instruction.as_deref(),
        Some("Arrive at destination")
    );

    assert_eq!(
        route.steps.len(),
        3,
        "the flat projection carries every leg's steps"
    );
    assert_eq!(
        route.geometry.points.len(),
        expected_shape.len() * 2,
        "the route geometry carries every section's shape"
    );
    assert_eq!(route.geometry.points[0], expected_shape[0]);
}

#[test]
fn the_reported_capabilities_match_what_here_honours() {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let router = HereRouter::new(Arc::new(HereClient::new(auth)));

    let capabilities = router.route_capabilities();

    assert_eq!(
        capabilities.supported_transport_modes,
        vec![
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
        ]
    );
    assert!(capabilities.supports_shortest_route_strategy);
}

#[tokio::test]
async fn a_shortest_strategy_asks_for_short_routing() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("routingMode", "short"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("routing_two_sections.json")),
        )
        .mount(&server)
        .await;

    let router = router_for(&server);
    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        route_strategy: Some(RouteStrategy::Shortest),
        ..Default::default()
    };

    router
        .calculate_route(&start, &end, &options)
        .await
        .expect("the short routing request must be served");
}

#[tokio::test]
async fn a_malformed_polyline_is_a_typed_error() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1500.0, "duration": 300.0 },
                "polyline": "!"
            }]
        }]
    });
    Mock::given(method("GET"))
        .and(path("/routes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let router = router_for(&server);
    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let error = router
        .calculate_route(&start, &end, &options)
        .await
        .expect_err("a broken shape must not decode to an empty polyline");

    match error {
        EveryMapError::ProviderError { provider, code, .. } => {
            assert_eq!(provider, "here");
            assert_eq!(code, "INVALID_POLYLINE");
        }
        other => panic!("expected the invalid-polyline provider error, got {other}"),
    }
}
