//! Contract tests of the Valhalla routing provider against a wiremock server
//! that replays committed fixtures. The harness asserts both directions: that
//! the POST carries the documented body, and that each response channel maps
//! to the contract's typed result or error.

use everymap_core::domains::routing::{
    AvoidType, ManeuverType, RouteOptions, RouteStrategy, Router, TransportMode, TurnDirection,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_valhalla::{ValhallaProviderConfiguration, ValhallaRouter};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// User agent the tests configure; also matched on the wire.
const TEST_USER_AGENT: &str = "everymap-valhalla-provider-tests";

/// Tolerance for the fixture's numeric fields.
const FIXTURE_NUMERIC_TOLERANCE: f64 = 1e-6;

fn router_for(base_url: String) -> ValhallaRouter {
    ValhallaRouter::new(ValhallaProviderConfiguration {
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

fn recorded_waypoints() -> Vec<Coordinate> {
    vec![
        coordinate(38.57657, -121.4936),
        coordinate(38.57887, -121.50797),
    ]
}

/// The exact body the default automobile request must post.
fn default_route_request_body() -> Value {
    json!({
        "locations": [
            {"lat": 38.57657, "lon": -121.4936},
            {"lat": 38.57887, "lon": -121.50797}
        ],
        "costing": "auto",
        "units": "kilometers",
        "directions_options": {"units": "kilometers"}
    })
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
    Mock::given(method("POST"))
        .and(path("/route"))
        .and(header("content-type", "application/json"))
        .and(header("user-agent", TEST_USER_AGENT))
        .and(body_json(default_route_request_body()))
        .respond_with(fixture_response("route_sacramento.json"))
        .expect(1)
        .mount(&server)
        .await;

    let response = router_for(server.uri())
        .calculate_route(
            &recorded_waypoints()[0],
            &recorded_waypoints()[1],
            &RouteOptions::default(),
        )
        .await
        .expect("the recorded trip must decode");
    server.verify().await;

    assert_eq!(response.routes.len(), 1);
    let route = &response.routes[0];
    assert_close(route.distance, 3_200.0);
    assert_close(route.duration, 420.0);
    assert_eq!(route.transport_mode, Some(TransportMode::Car));
    assert_eq!(route.legs.len(), 1);
    assert_eq!(route.steps.len(), 5);

    let leg = &route.legs[0];
    assert_close(leg.distance, 3_200.0);
    assert_close(leg.duration, 420.0);
    assert_eq!(leg.steps.len(), 5);
    assert_eq!(leg.shape.len(), 12);
    assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
    assert_eq!(
        leg.steps[1].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Right))
    );
    assert_eq!(leg.steps[2].maneuver_type, Some(ManeuverType::Continue));
    assert_eq!(
        leg.steps[3].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Left))
    );
    assert_eq!(leg.steps[4].maneuver_type, Some(ManeuverType::Arrival));
    assert_eq!(
        leg.steps[0].instruction.as_deref(),
        Some("Drive west on N Street.")
    );
    assert_eq!(
        leg.steps[1].instruction.as_deref(),
        Some("Turn right onto 10th Street.")
    );
    assert_eq!(
        leg.steps[1].speech_instruction.as_deref(),
        Some("Turn right onto Tenth Street.")
    );
    assert_eq!(leg.steps[0].shape.len(), 3);
    assert_eq!(leg.steps[4].shape.len(), 3);

    let bounding_box = route.bounding_box.expect("the route has a bounding box");
    assert_close(bounding_box.south_west.lat, 38.57652);
    assert_close(bounding_box.south_west.lng, -121.50797);
    assert_close(bounding_box.north_east.lat, 38.57887);
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
    assert!(capabilities.supports_shortest_route_strategy);
}

#[tokio::test]
async fn ordered_waypoints_post_every_location_in_order() {
    let server = MockServer::start().await;
    let waypoints = [
        coordinate(38.57657, -121.4936),
        coordinate(38.5775, -121.499),
        coordinate(38.57887, -121.50797),
    ];
    Mock::given(method("POST"))
        .and(path("/route"))
        .and(body_json(json!({
            "locations": [
                {"lat": 38.57657, "lon": -121.4936},
                {"lat": 38.5775, "lon": -121.499},
                {"lat": 38.57887, "lon": -121.50797}
            ],
            "costing": "auto",
            "units": "kilometers",
            "directions_options": {"units": "kilometers"}
        })))
        .respond_with(fixture_response("route_sacramento.json"))
        .expect(1)
        .mount(&server)
        .await;

    router_for(server.uri())
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .expect("the recorded trip must decode");
    server.verify().await;
}

#[tokio::test]
async fn shortest_strategy_and_avoidance_ride_costing_options() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .and(body_json(json!({
            "locations": [
                {"lat": 38.57657, "lon": -121.4936},
                {"lat": 38.57887, "lon": -121.50797}
            ],
            "costing": "auto",
            "units": "kilometers",
            "directions_options": {"units": "kilometers"},
            "costing_options": {
                "auto": {
                    "shortest": true,
                    "use_tolls": 0.0,
                    "use_highways": 0.0,
                    "use_ferry": 0.0,
                    "use_tracks": 0.0
                }
            }
        })))
        .respond_with(fixture_response("route_sacramento.json"))
        .expect(1)
        .mount(&server)
        .await;

    let options = RouteOptions {
        route_strategy: Some(RouteStrategy::Shortest),
        avoid: vec![
            AvoidType::Tolls,
            AvoidType::Highways,
            AvoidType::Ferries,
            AvoidType::DirtRoads,
        ],
        ..RouteOptions::default()
    };
    router_for(server.uri())
        .calculate_route_with_waypoints(&recorded_waypoints(), &options)
        .await
        .expect("the recorded trip must decode");
    server.verify().await;
}

#[tokio::test]
async fn no_route_answer_surfaces_the_service_code_and_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(fixture("route_no_route_error.json")),
        )
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&recorded_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a refusal must surface");
    match error {
        EveryMapError::ProviderError {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "valhalla");
            assert_eq!(code, "440");
            assert_eq!(message, "No route found between origin and destination");
        }
        other => panic!("expected a provider error, got {other:?}"),
    }
}

#[tokio::test]
async fn http_server_failure_surfaces_the_http_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&recorded_waypoints(), &RouteOptions::default())
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
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"trip": {"summary": }}"#))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&recorded_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a malformed body must surface");
    assert!(matches!(error, EveryMapError::SerializationError { .. }));
}

#[tokio::test]
async fn a_valid_body_without_a_trip_surfaces_a_serialization_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"unexpected": "payload"})))
        .mount(&server)
        .await;

    let error = router_for(server.uri())
        .calculate_route_with_waypoints(&recorded_waypoints(), &RouteOptions::default())
        .await
        .expect_err("a body without a trip must surface");
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
