//! Contract tests of the HERE traffic provider against committed flow and
//! incident fixtures.
//!
//! The fixtures under `tests/fixtures` are hand-authored minimal payloads in
//! the shapes HERE Traffic API v7 documents. The tests assert that the
//! contract's radius, which is stated in metres, becomes a correspondingly
//! small degree box, that a caller-supplied bounding box drives both traffic
//! endpoints, that an incident failure is returned instead of swallowed, and
//! that a request without any area is refused.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::error::EveryMapError;
use everymap_core::types::{BoundingBox, Coordinate};
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::traffic::{HereFlowOptions, HereTraffic};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// Directory holding the committed traffic payloads.
const FIXTURES_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Coordinate the traffic queries are centred on.
const BERLIN_LATITUDE: f64 = 52.52;
const BERLIN_LONGITUDE: f64 = 13.405;

/// Metres in one degree of latitude at the WGS 84 mean radius.
const METRES_PER_DEGREE_LATITUDE: f64 = 111_320.0;

/// Radius the unit tests request, in the contract's metres.
const FIVE_KILOMETRE_RADIUS_METRES: f64 = 5_000.0;

/// Allowed drift when a formatted query value is parsed back.
const BOUNDING_BOX_TOLERANCE_DEGREES: f64 = 1e-9;

/// Reads one committed fixture.
fn fixture(name: &str) -> serde_json::Value {
    let fixture_path = format!("{FIXTURES_DIRECTORY}/{name}");
    serde_json::from_str(
        &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
    )
    .expect("the fixture must be valid JSON")
}

/// Reads one query parameter from a recorded request.
fn query_value(request: &Request, key: &str) -> String {
    request
        .url
        .query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
        .expect("the request carries the query parameter")
}

/// Parses a HERE `bbox:west,south,east,north` filter.
fn parse_here_bounding_box(filter: &str) -> (f64, f64, f64, f64) {
    let coordinates = filter
        .strip_prefix("bbox:")
        .expect("an in filter names a bbox")
        .split(',')
        .map(|part| part.parse::<f64>().expect("a bbox part is numeric"))
        .collect::<Vec<f64>>();
    assert_eq!(coordinates.len(), 4, "a bbox has four coordinates");
    (
        coordinates[0],
        coordinates[1],
        coordinates[2],
        coordinates[3],
    )
}

fn traffic_provider_for(server: &MockServer) -> HereTraffic {
    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    HereTraffic::with_base_url(client, server.uri())
}

#[tokio::test]
async fn test_traffic_contract() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_berlin.json")))
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);

    let coordinate = Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap();
    let options = TrafficOptions {
        radius: Some(FIVE_KILOMETRE_RADIUS_METRES),
        ..TrafficOptions::default()
    };

    let response = traffic_provider
        .get_traffic(&coordinate, &options)
        .await
        .unwrap();

    assert_eq!(response.flows.len(), 1);
    assert_eq!(response.flows[0].jam_factor.unwrap(), 2.5);
    assert!(response.incidents.is_empty());
}

#[tokio::test]
async fn a_metre_radius_becomes_a_small_degree_box() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_berlin.json")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/incidents"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_incidents_berlin.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);
    let coordinate = Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap();
    let options = TrafficOptions {
        radius: Some(FIVE_KILOMETRE_RADIUS_METRES),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };

    traffic_provider
        .get_traffic(&coordinate, &options)
        .await
        .unwrap();
    server.verify().await;

    let requests = server
        .received_requests()
        .await
        .expect("wiremock records requests");
    let flow_area = query_value(
        requests
            .iter()
            .find(|request| request.url.path() == "/flow")
            .expect("the flow endpoint is called"),
        "in",
    );
    let incident_area = query_value(
        requests
            .iter()
            .find(|request| request.url.path() == "/incidents")
            .expect("the incident endpoint is called"),
        "in",
    );
    assert_eq!(
        flow_area, incident_area,
        "both endpoints must query the same ground"
    );

    let (west, south, east, north) = parse_here_bounding_box(&flow_area);
    let latitude_span = north - south;
    let expected_latitude_span = 2.0 * FIVE_KILOMETRE_RADIUS_METRES / METRES_PER_DEGREE_LATITUDE;
    assert!(
        (latitude_span - expected_latitude_span).abs() < BOUNDING_BOX_TOLERANCE_DEGREES,
        "a 5 km radius must span about {expected_latitude_span} degrees of latitude, got {latitude_span}"
    );
    assert!(
        latitude_span < 1.0,
        "a 5 km radius must not become a global box, got {latitude_span} degrees"
    );
    assert!(
        ((south + north) / 2.0 - BERLIN_LATITUDE).abs() < BOUNDING_BOX_TOLERANCE_DEGREES,
        "the box must stay centred on the requested latitude"
    );
    assert!(
        ((west + east) / 2.0 - BERLIN_LONGITUDE).abs() < BOUNDING_BOX_TOLERANCE_DEGREES,
        "the box must stay centred on the requested longitude"
    );
}

#[tokio::test]
async fn a_caller_bounding_box_drives_both_traffic_endpoints() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/flow"))
        .and(query_param("in", "bbox:13.3,52.4,13.5,52.6"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_berlin.json")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/incidents"))
        .and(query_param("in", "bbox:13.3,52.4,13.5,52.6"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_incidents_berlin.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);
    let bounding_box = BoundingBox::new(
        Coordinate::new(52.6, 13.5).unwrap(),
        Coordinate::new(52.4, 13.3).unwrap(),
    );
    let options = TrafficOptions {
        bounding_box: Some(bounding_box),
        // A radius beside a box must not win over the caller's viewport.
        radius: Some(FIVE_KILOMETRE_RADIUS_METRES),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };

    let response = traffic_provider
        .get_traffic(
            &Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap(),
            &options,
        )
        .await
        .unwrap();
    server.verify().await;

    assert_eq!(response.flows.len(), 1);
    assert_eq!(response.incidents.len(), 1);
}

#[tokio::test]
async fn an_incident_failure_surfaces_instead_of_being_swallowed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("traffic_flow_berlin.json")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/incidents"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad incident query"))
        .expect(1)
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);
    let options = TrafficOptions {
        radius: Some(FIVE_KILOMETRE_RADIUS_METRES),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };

    let error = traffic_provider
        .get_traffic(
            &Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap(),
            &options,
        )
        .await
        .expect_err("an incident failure must reach the caller");

    match error {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 400),
        other => panic!("expected the incident HTTP failure, got {other}"),
    }
}

#[tokio::test]
async fn a_request_without_an_area_is_refused() {
    let server = MockServer::start().await;
    let traffic_provider = traffic_provider_for(&server);

    let error = traffic_provider
        .get_traffic(
            &Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap(),
            &TrafficOptions::default(),
        )
        .await
        .expect_err("the service has no default area to guess");

    assert!(
        matches!(&error, EveryMapError::ValidationError(_)),
        "expected a validation error, got {error}"
    );
    assert!(
        server
            .received_requests()
            .await
            .expect("wiremock records requests")
            .is_empty(),
        "a query without an area must not reach the network"
    );
}

#[tokio::test]
async fn test_traffic_flow_with_rich_types() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "location": {
                    "description": "A100 Motorway",
                    "length": 2500.0,
                    "functionalClass": 1
                },
                "currentFlow": {
                    "speed": 85.0,
                    "speedUncapped": 88.0,
                    "freeFlow": 130.0,
                    "jamFactor": 1.2,
                    "confidence": 0.95,
                    "traversability": "open"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);

    let response = traffic_provider
        .get_flow(
            Coordinate::new(BERLIN_LATITUDE, BERLIN_LONGITUDE).unwrap(),
            &HereFlowOptions::default(),
        )
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    let item = &response.results[0];
    assert_eq!(item.current_flow.speed.unwrap(), 85.0);
    assert_eq!(item.current_flow.jam_factor.unwrap(), 1.2);
    assert_eq!(item.current_flow.free_flow.unwrap(), 130.0);
    assert_eq!(item.location.description.as_ref().unwrap(), "A100 Motorway");
    assert_eq!(item.current_flow.traversability.as_ref().unwrap(), "open");
}

#[tokio::test]
async fn test_traffic_incidents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/incidents"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("traffic_incidents_berlin.json")),
        )
        .mount(&server)
        .await;

    let traffic_provider = traffic_provider_for(&server);

    use everymap_providers_here::domain::traffic::HereIncidentsOptions;
    let response = traffic_provider
        .get_incidents(&HereIncidentsOptions {
            in_filter: Some("bbox:13.0,52.0,14.0,53.0".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    let incident = &response.results[0].incident;
    assert_eq!(incident.id.as_ref().unwrap(), "INC_123");
    assert_eq!(incident.incident_type.as_ref().unwrap(), "accident");
    assert_eq!(incident.criticality.as_ref().unwrap(), "major");
}
