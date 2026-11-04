//! Contract tests of the MapQuest router against committed fixtures.
//!
//! The tests assert the documented query shape on the wire and the projected
//! route: legs, maneuvers, end-anchored instruction text, shapes, bounding box
//! and the capabilities the service honours.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::routing::{
    AvoidType, ManeuverType, RouteOptions, RouteStrategy, Router, TransportMode, TurnDirection,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::MapQuestRouter;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Latitude/longitude tolerance for the fixture's numeric fields.
const NUMERIC_TOLERANCE: f64 = 1e-6;

fn coordinate(latitude: f64, longitude: f64) -> Coordinate {
    Coordinate::new(latitude, longitude).unwrap()
}

fn sacramento_waypoints() -> [Coordinate; 2] {
    [
        coordinate(38.57693, -121.49492),
        coordinate(38.57874, -121.50745),
    ]
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < NUMERIC_TOLERANCE,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn a_recorded_route_projects_legs_maneuvers_text_and_shapes() {
    let server = MockServer::start().await;
    let waypoints = sacramento_waypoints();
    Mock::given(method("GET"))
        .and(path("/directions/v2/route"))
        .and(query_param("from", "38.57693,-121.49492"))
        .and(query_param("to", "38.57874,-121.50745"))
        .and(query_param("unit", "k"))
        .and(query_param("routeType", "fastest"))
        .and(query_param("shapeFormat", "raw"))
        .and(query_param("fullShape", "true"))
        .and(query_param("narrativeType", "text"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("directions_route_two_waypoints.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let router = MapQuestRouter::new(client_for(&server));
    let response = router
        .calculate_route(&waypoints[0], &waypoints[1], &RouteOptions::default())
        .await
        .expect("the recorded route must project");
    server.verify().await;

    assert_eq!(response.routes.len(), 1);
    let route = &response.routes[0];
    assert!((route.distance - 1700.0).abs() < 50.0);
    assert_close(route.duration, 235.0);
    assert_eq!(route.geometry.points.len(), 45);
    assert_eq!(route.transport_mode, Some(TransportMode::Car));
    assert_eq!(route.legs.len(), 1);
    assert_eq!(route.steps.len(), 6);

    let leg = &route.legs[0];
    assert_eq!(leg.steps.len(), 6);
    assert!(!leg.shape.is_empty());
    assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
    assert_eq!(leg.steps[5].maneuver_type, Some(ManeuverType::Arrival));
    assert_eq!(
        leg.steps[1].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Left))
    );
    assert_eq!(
        leg.steps[3].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Right))
    );
    assert_eq!(
        leg.steps[4].maneuver_type,
        Some(ManeuverType::Turn(TurnDirection::Right))
    );
    assert_eq!(
        leg.steps[0].instruction.as_deref(),
        Some("Turn left onto L St. Go for 864 m.")
    );
    assert_eq!(
        leg.steps[0].speech_instruction.as_deref(),
        Some("Turn left onto L St. Go for 864 m.")
    );
    assert_eq!(
        leg.steps[5].instruction.as_deref(),
        Some("Arrive at Front St.")
    );
    for step in &leg.steps {
        assert!(!step.shape.is_empty(), "every recorded step has a shape");
        assert!(step.lane_guidances.is_empty());
        assert!(step.start_coordinate.is_some());
        assert!(step.end_coordinate.is_some());
    }

    let bounding_box = route.bounding_box.expect("the route has a bounding box");
    assert_close(bounding_box.north_east.lat, 38.58060000000001);
    assert_close(bounding_box.north_east.lng, -121.49439);
    assert_close(bounding_box.south_west.lat, 38.57693);
    assert_close(bounding_box.south_west.lng, -121.50780000000005);
}

#[tokio::test]
async fn the_router_reports_the_modes_and_strategy_the_service_honours() {
    let server = MockServer::start().await;
    let router = MapQuestRouter::new(client_for(&server));
    let capabilities = router.route_capabilities();
    assert_eq!(
        capabilities.supported_transport_modes,
        vec![
            TransportMode::Car,
            TransportMode::Bicycle,
            TransportMode::Pedestrian
        ]
    );
    assert!(capabilities.supports_shortest_route_strategy);
    assert!(!capabilities.supports_transport_mode(TransportMode::Truck));
}

#[tokio::test]
async fn ordered_waypoints_send_one_to_value_per_stop_in_request_order() {
    let server = MockServer::start().await;
    let waypoints = [
        coordinate(38.57693, -121.49492),
        coordinate(38.5775, -121.499),
        coordinate(38.57874, -121.50745),
    ];
    Mock::given(method("GET"))
        .and(path("/directions/v2/route"))
        .and(query_param("from", "38.57693,-121.49492"))
        .and(query_param("to", "38.5775,-121.499"))
        .and(query_param("to", "38.57874,-121.50745"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("directions_route_two_waypoints.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let response = MapQuestRouter::new(client_for(&server))
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .expect("the recorded route must project");
    server.verify().await;
    assert_eq!(response.routes.len(), 1);
}

#[tokio::test]
async fn the_strategy_and_avoidance_options_reach_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/directions/v2/route"))
        .and(query_param("routeType", "shortest"))
        .and(query_param("avoids", "Toll Road,Ferry"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("directions_route_two_waypoints.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        route_strategy: Some(RouteStrategy::Shortest),
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        ..RouteOptions::default()
    };
    MapQuestRouter::new(client_for(&server))
        .calculate_route_with_waypoints(&sacramento_waypoints(), &options)
        .await
        .expect("the recorded route must project");
    server.verify().await;
}

#[tokio::test]
async fn a_bicycle_request_selects_the_bicycle_route_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/directions/v2/route"))
        .and(query_param("routeType", "bicycle"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("directions_route_two_waypoints.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let options = RouteOptions {
        transport_mode: Some(TransportMode::Bicycle),
        route_strategy: Some(RouteStrategy::Shortest),
        ..RouteOptions::default()
    };
    MapQuestRouter::new(client_for(&server))
        .calculate_route_with_waypoints(&sacramento_waypoints(), &options)
        .await
        .expect("the recorded route must project");
    server.verify().await;
}

#[tokio::test]
async fn fewer_than_two_waypoints_is_rejected_before_any_request() {
    let server = MockServer::start().await;
    let router = MapQuestRouter::new(client_for(&server));
    for waypoints in [Vec::new(), vec![coordinate(38.57693, -121.49492)]] {
        let error = router
            .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
            .await
            .expect_err("a route needs at least two waypoints");
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("at least an origin"), "detail was {detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }
}
