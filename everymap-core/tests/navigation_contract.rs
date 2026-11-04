//! Contract tests for the navigation-grade routing surface.
//!
//! The recorded payload is served from `tests/fixtures/navigation_route.json`
//! and decoded into the core records; no test reaches the network.

use async_trait::async_trait;
use everymap_core::domains::routing::{
    LaneGuidance, ManeuverType, RouteCapabilities, RouteOptions, RouteResponse, Router,
    TurnDirection, INTERMEDIATE_WAYPOINTS_DOMAIN,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use std::sync::Mutex;

/// Recorded navigation-grade route through two consecutive waypoint pairs.
const NAVIGATION_ROUTE_JSON: &str = include_str!("fixtures/navigation_route.json");

fn coordinate(lat: f64, lng: f64) -> Coordinate {
    Coordinate::new(lat, lng).unwrap()
}

/// Serves the recorded route and remembers the waypoints it was asked for.
struct RecordedNavigationRouter {
    response: RouteResponse,
    seen_waypoints: Mutex<Vec<Coordinate>>,
}

impl RecordedNavigationRouter {
    fn new(response: RouteResponse) -> Self {
        Self {
            response,
            seen_waypoints: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Router for RecordedNavigationRouter {
    async fn calculate_route(
        &self,
        _start: &Coordinate,
        _end: &Coordinate,
        _options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        Err(EveryMapError::ValidationError(
            "this test router only serves ordered waypoints".to_string(),
        ))
    }

    async fn calculate_route_with_waypoints(
        &self,
        waypoints: &[Coordinate],
        _options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        *self.seen_waypoints.lock().unwrap() = waypoints.to_vec();
        Ok(self.response.clone())
    }

    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                everymap_core::domains::routing::TransportMode::Car,
                everymap_core::domains::routing::TransportMode::Truck,
            ],
            supports_shortest_route_strategy: false,
        }
    }
}

/// Implements only the original two-endpoint method, so it also exercises the
/// default bodies of the navigation-grade additions.
struct TwoEndpointOnlyRouter {
    calls: Mutex<usize>,
}

impl TwoEndpointOnlyRouter {
    fn new() -> Self {
        Self {
            calls: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Router for TwoEndpointOnlyRouter {
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        _options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        *self.calls.lock().unwrap() += 1;
        Ok(RouteResponse {
            routes: vec![everymap_core::domains::routing::RouteResult {
                distance: start.lat.abs() + end.lat.abs(),
                duration: 0.0,
                geometry: everymap_core::types::Polyline::new(vec![*start, *end]),
                transport_mode: None,
                steps: Vec::new(),
                bounding_box: None,
                raw: None,
                legs: Vec::new(),
            }],
        })
    }
}

#[tokio::test]
async fn recorded_route_preserves_ordered_waypoints_and_navigation_fields() {
    let response: RouteResponse = serde_json::from_str(NAVIGATION_ROUTE_JSON).unwrap();
    let router = RecordedNavigationRouter::new(response);
    let waypoints = vec![
        coordinate(52.5219, 13.4132),
        coordinate(52.5163, 13.3777),
        coordinate(52.5096, 13.3760),
    ];

    let routed = router
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .unwrap();

    assert_eq!(
        *router.seen_waypoints.lock().unwrap(),
        waypoints,
        "waypoints reach the provider in request order"
    );

    let route = &routed.routes[0];
    assert_eq!(route.legs.len(), 2, "one leg per consecutive waypoint pair");
    assert_eq!(route.legs[0].distance, 1800.0);
    assert_eq!(route.legs[1].shape.len(), 2);

    let departure = &route.legs[0].steps[0];
    assert_eq!(departure.maneuver_type, Some(ManeuverType::Departure));
    assert_eq!(
        departure.speech_instruction.as_deref(),
        Some("Head west on Karl-Liebknecht Street.")
    );

    let roundabout = &route.legs[1].steps[0];
    assert_eq!(
        roundabout.maneuver_type,
        Some(ManeuverType::Roundabout(2)),
        "the exit index survives the recorded payload"
    );
    assert_eq!(
        roundabout.lane_guidances,
        vec![LaneGuidance {
            lane_index: 1,
            permitted_directions: vec![TurnDirection::Left, TurnDirection::UTurn],
            is_recommended: true,
        }]
    );
    assert_eq!(
        route.legs[1].steps[1].maneuver_type,
        Some(ManeuverType::Arrival)
    );
}

#[tokio::test]
async fn recorded_route_roundtrips_through_json_with_stable_maneuver_labels() {
    let response: RouteResponse = serde_json::from_str(NAVIGATION_ROUTE_JSON).unwrap();
    let serialized = serde_json::to_string(&response).unwrap();

    assert!(serialized.contains("\"roundabout-2\""));
    assert!(serialized.contains("\"u_turn\""));
    assert!(serialized.contains("\"speech_instruction\""));

    let decoded: RouteResponse = serde_json::from_str(&serialized).unwrap();
    let maneuver = decoded.routes[0].legs[1].steps[0].maneuver_type;
    assert_eq!(maneuver, Some(ManeuverType::Roundabout(2)));
}

#[tokio::test]
async fn default_waypoint_method_delegates_a_two_endpoint_request() {
    let router = TwoEndpointOnlyRouter::new();
    let start = coordinate(52.5, 13.4);
    let end = coordinate(52.6, 13.5);

    let response = router
        .calculate_route_with_waypoints(&[start, end], &RouteOptions::default())
        .await
        .unwrap();

    assert_eq!(*router.calls.lock().unwrap(), 1);
    assert_eq!(response.routes.len(), 1);
}

#[tokio::test]
async fn default_waypoint_method_reports_intermediate_waypoints_as_unsupported() {
    let router = TwoEndpointOnlyRouter::new();
    let waypoints = [
        coordinate(52.5, 13.4),
        coordinate(52.6, 13.5),
        coordinate(52.7, 13.6),
    ];

    let error = router
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        EveryMapError::UnsupportedDomain { ref domain, .. }
            if domain == INTERMEDIATE_WAYPOINTS_DOMAIN
    ));
    assert_eq!(*router.calls.lock().unwrap(), 0);
}

#[tokio::test]
async fn default_waypoint_method_rejects_fewer_than_two_waypoints() {
    let router = TwoEndpointOnlyRouter::new();
    let error = router
        .calculate_route_with_waypoints(&[coordinate(52.5, 13.4)], &RouteOptions::default())
        .await
        .unwrap_err();

    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn default_route_capabilities_cover_every_mode_and_strategy() {
    let router = TwoEndpointOnlyRouter::new();
    let capabilities = router.route_capabilities();
    assert_eq!(capabilities, RouteCapabilities::all_supported());
    assert!(capabilities.supports_shortest_route_strategy);
}

#[test]
fn provider_capability_override_is_honoured() {
    let response: RouteResponse = serde_json::from_str(NAVIGATION_ROUTE_JSON).unwrap();
    let router = RecordedNavigationRouter::new(response);
    let capabilities = router.route_capabilities();

    assert!(!capabilities.supports_shortest_route_strategy);
    assert!(
        capabilities.supports_transport_mode(everymap_core::domains::routing::TransportMode::Car)
    );
    assert!(!capabilities
        .supports_transport_mode(everymap_core::domains::routing::TransportMode::Pedestrian));
}
