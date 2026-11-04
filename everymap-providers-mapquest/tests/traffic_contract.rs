//! Contract tests of the MapQuest traffic provider against committed flow and
//! incident fixtures.
//!
//! The tests assert that a requested bounding box drives both the flow view
//! and the incident query, that flow geometry and congestion levels are
//! projected, that incidents carry their type and severity, and that the
//! incident endpoint is only called when the caller asks for incidents.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::traffic::{
    CongestionLevel, IncidentSeverity, TrafficOptions, TrafficProvider,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::{BoundingBox, Coordinate};
use everymap_providers_mapquest::MapQuestTrafficProvider;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Ground the flow and incident requests must cover.
fn new_york_box() -> BoundingBox {
    BoundingBox::new(
        Coordinate::new(40.7, -73.99).unwrap(),
        Coordinate::new(40.6, -74.01).unwrap(),
    )
}

#[tokio::test]
async fn flow_and_incidents_honour_the_requested_bounding_box() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/traffic/v2/flow"))
        .and(query_param("projection", "merc"))
        .and(query_param("minSeverity", "green"))
        .and(query_param("maxLines", "1000"))
        .and(query_param("outFormat", "json"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_new_york.json")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/traffic/v2/incidents"))
        .and(query_param("boundingBox", "40.7,-74.01,40.6,-73.99"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_incidents_denver.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestTrafficProvider::new(client_for(&server));
    let options = TrafficOptions {
        bounding_box: Some(new_york_box()),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };
    let response = provider
        .get_traffic(&Coordinate::new(40.65, -74.0).unwrap(), &options)
        .await
        .expect("the recorded traffic must project");
    server.verify().await;

    assert_eq!(response.flows.len(), 4);
    let levels = response
        .flows
        .iter()
        .map(|flow| flow.congestion_level)
        .collect::<Vec<Option<CongestionLevel>>>();
    assert_eq!(
        levels,
        vec![
            Some(CongestionLevel::FreeFlow),
            Some(CongestionLevel::Slow),
            Some(CongestionLevel::Congested),
            Some(CongestionLevel::Blocked),
        ]
    );
    let first_flow = &response.flows[0];
    assert_eq!(first_flow.geometry.len(), 23);
    assert!((first_flow.geometry[0].lat - 40.63701).abs() < 1e-9);
    assert!((first_flow.geometry[0].lng - (-74.01926)).abs() < 1e-9);
    assert!(first_flow.speed.is_none());

    assert_eq!(response.incidents.len(), 5);
    let first_incident = &response.incidents[0];
    assert_eq!(first_incident.incident_type.as_deref(), Some("accident"));
    assert_eq!(first_incident.severity, Some(IncidentSeverity::Moderate));
    assert_eq!(first_incident.description.as_deref(), Some("Closed"));
    assert_eq!(
        first_incident.start_time.as_deref(),
        Some("2026-10-26T12:03:28")
    );
    assert!(
        first_incident.start_coordinate.is_some(),
        "an incident carries the position the service reported"
    );
}

#[tokio::test]
async fn incidents_are_not_requested_when_the_caller_declines_them() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/traffic/v2/flow"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_new_york.json")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/traffic/v2/incidents"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_incidents_denver.json")),
        )
        .expect(0)
        .mount(&server)
        .await;

    let provider = MapQuestTrafficProvider::new(client_for(&server));
    let options = TrafficOptions {
        bounding_box: Some(new_york_box()),
        include_incidents: Some(false),
        ..TrafficOptions::default()
    };
    let response = provider
        .get_traffic(&Coordinate::new(40.65, -74.0).unwrap(), &options)
        .await
        .expect("the recorded flow must project");
    server.verify().await;
    assert_eq!(response.flows.len(), 4);
    assert!(response.incidents.is_empty());
}

#[tokio::test]
async fn a_box_beyond_the_service_area_limit_is_rejected_without_a_request() {
    let server = MockServer::start().await;
    let provider = MapQuestTrafficProvider::new(client_for(&server));
    let options = TrafficOptions {
        bounding_box: Some(BoundingBox::new(
            Coordinate::new(41.0, -73.0).unwrap(),
            Coordinate::new(39.9, -74.0).unwrap(),
        )),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };
    let error = provider
        .get_traffic(&Coordinate::new(40.45, -73.5).unwrap(), &options)
        .await
        .expect_err("a box beyond the area limit must be refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}
