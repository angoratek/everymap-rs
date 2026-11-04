//! Contract tests of the OSRM routing provider against a wiremock server that
//! replays committed fixtures. The harness asserts both directions: that the
//! request carries the documented path and query parameters, and that each
//! response channel maps to the contract's typed result or error.

use everymap_core::domains::routing::{
    AvoidType, ManeuverType, RouteOptions, RouteStrategy, Router, TransportMode, TurnDirection,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};
use serde_json::Value;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// User agent the tests configure; also matched on the wire.
const TEST_USER_AGENT: &str = "everymap-osrm-provider-tests";

/// Tolerance for the fixture's numeric fields.
const FIXTURE_NUMERIC_TOLERANCE: f64 = 1e-6;

fn router_for(base_url: String) -> OsrmRouter {
    OsrmRouter::new(OsrmProviderConfiguration {
        base_url,
        user_agent: TEST_USER_AGENT.to_string(),
    })
}

fn fixture(name: &str) -> Value {
    let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(
        &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
    )
    .expect("the fixture must be valid JSON")
}

fn fixture_response(name: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(fixture(name))
}

fn coordinate(latitude: f64, longitude: f64) -> Coordinate {
    Coordinate::new(latitude, longitude).unwrap()
}

fn sacramento_waypoints() -> Vec<Coordinate> {
    vec![
        coordinate(38.57657, -121.4936),
        coordinate(38.5781, -121.50555),
    ]
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < FIXTURE_NUMERIC_TOLERANCE,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn successful_route_decodes_legs_steps_instructions_shapes_and_capabilities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/route/v1/driving/-121.4936,38.57657;-121.50555,38.5781",
        ))
        .and(query_param("overview", "full"))
        .and(query_param("geometries", "geojson"))
        .and(query_param("steps", "true"))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(fixture_response("sacramento_route.json"))
        .expect(1)
        .mount(&server)
        .await;

    let response = router_for(server.uri())
        .calculate_route(
            &sacramento_waypoints()[0],
            &sacramento_waypoints()[1],
            &RouteOptions::default(),
        )
        .await
        .expect("the recorded route must decode");
    server.verify().await;

    assert_eq!(response.routes.len(), 1);
    let route = &response.routes[0];
    assert_close(route.distance, 1302.3);
    assert_close(route.duration, 157.1);
    assert_eq!(route.geometry.points.len(), 9);
    assert_eq!(route.transport_mode, Some(TransportMode::Car));
    assert_eq!(route.legs.len(), 1);
    assert_eq!(route.steps.len(), 4);

    let leg = &route.legs[0];
    assert_close(leg.distance, 1302.3);
    assert_close(leg.duration, 157.1);
    assert_eq!(leg.steps.len(), 4);
    assert!(!leg.shape.is_empty());
    assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
    assert_eq!(
        leg.steps[1].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Left))
    );
    assert_eq!(
        leg.steps[2].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Right))
    );
    assert_eq!(leg.steps[3].maneuver_type, Some(ManeuverType::Arrival));
    assert_eq!(
        leg.steps[1].instruction.as_deref(),
        Some("Turn left onto L Street")
    );
    assert_eq!(
        leg.steps[1].speech_instruction.as_deref(),
        Some("Turn left onto L Street")
    );
    for step in &leg.steps {
        assert!(!step.shape.is_empty(), "every recorded step has a shape");
        assert!(step.lane_guidances.is_empty());
    }

    let bounding_box = route.bounding_box.expect("the route has a bounding box");
    assert_close(bounding_box.south_west.lat, 38.57657);
    assert_close(bounding_box.south_west.lng, -121.50555);
    assert_close(bounding_box.north_east.lat, 38.5781);
    assert_close(bounding_box.north_east.lng, -121.4936);

    let capabilities = router_for(server.uri()).route_capabilities();
    assert_eq!(
        capabilities.supported_transport_modes,
        vec![
            TransportMode::Car,
            TransportMode::Bicycle,
            TransportMode::Pedestrian
        ]
    );
    assert!(!capabilities.supports_shortest_route_strategy);
}

#[tokio::test]
async fn ordered_waypoints_send_one_leg_per_pair_in_request_order() {
    let server = MockServer::start().await;
    let waypoints = [
        coordinate(38.57657, -121.4936),
        coordinate(38.5775, -121.499),
        coordinate(38.5781, -121.50555),
    ];
    Mock::given(method("GET"))
        .and(path(
            "/route/v1/driving/-121.4936,38.57657;-121.499,38.5775;-121.50555,38.5781",
        ))
        .respond_with(fixture_response("sacramento_route.json"))
        .expect(1)
        .mount(&server)
        .await;

    let response = router_for(server.uri())
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .expect("the recorded route must decode");
    server.verify().await;
    assert_eq!(response.routes.len(), 1);
}

#[tokio::test]
async fn ferry_route_classifies_the_crossing_step_as_a_ferry() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/route/v1/driving/-122.3937,37.7955;-122.2711,37.8044",
        ))
        .respond_with(fixture_response("san_francisco_oakland_ferry_route.json"))
        .expect(1)
        .mount(&server)
        .await;

    let response = router_for(server.uri())
        .calculate_route(
            &coordinate(37.7955, -122.3937),
            &coordinate(37.8044, -122.2711),
            &RouteOptions::default(),
        )
        .await
        .expect("the recorded ferry route must decode");
    server.verify().await;

    let steps = &response.routes[0].legs[0].steps;
    assert_eq!(steps.len(), 3);
    let ferry_step = steps
        .iter()
        .find(|step| step.maneuver_type == Some(ManeuverType::Ferry))
        .expect("the crossing step must classify as a ferry maneuver");
    assert_eq!(ferry_step.instruction.as_deref(), Some("Take the ferry"));
}

#[tokio::test]
async fn avoidance_options_become_one_exclude_query_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/route/v1/driving/-121.4936,38.57657;-121.50555,38.5781",
        ))
        .and(query_param("exclude", "toll,ferry"))
        .respond_with(fixture_response("sacramento_route.json"))
        .expect(1)
        .mount(&server)
        .await;

    let options = RouteOptions {
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        route_strategy: Some(RouteStrategy::Shortest),
        ..RouteOptions::default()
    };
    router_for(server.uri())
        .calculate_route_with_waypoints(&sacramento_waypoints(), &options)
        .await
        .expect("the strategy must not change the request");
    server.verify().await;
}

#[tokio::test]
async fn no_route_answer_surfaces_the_service_code_and_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(400).set_body_json(fixture("no_route.json")))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&sacramento_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a refusal must surface");
    match error {
        EveryMapError::ProviderError {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "osrm");
            assert_eq!(code, "NoRoute");
            assert_eq!(message, "Impossible route between points");
        }
        other => panic!("expected a provider error, got {other:?}"),
    }
}

#[tokio::test]
async fn http_server_failure_surfaces_the_http_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upstream failure"))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&sacramento_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a server failure must surface");
    match error {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 500),
        other => panic!("expected an HTTP error, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_success_body_surfaces_a_serialization_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&sacramento_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a malformed body must surface");
    assert!(matches!(error, EveryMapError::SerializationError { .. }));
}

#[tokio::test]
async fn fewer_than_two_waypoints_is_rejected_before_any_request() {
    // The base URL names no reachable server: a validation error proves the
    // guard runs before a request is sent.
    let router = router_for("http://127.0.0.1:1".to_string());
    for waypoints in [Vec::new(), vec![coordinate(38.57657, -121.4936)]] {
        let error = router
            .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
            .await
            .expect_err("a route needs two waypoints");
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("at least 2"), "detail was {detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }
}
