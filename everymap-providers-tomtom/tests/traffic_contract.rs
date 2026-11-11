use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::traffic::{IncidentSeverity, TrafficOptions, TrafficProvider};
use everymap_core::types::{BoundingBox, Coordinate};
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomTraffic;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_traffic_flow_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "flowSegmentData": {
            "frc": "FRC0",
            "currentSpeed": 65,
            "freeFlowSpeed": 80,
            "currentTravelTime": 120,
            "freeFlowTravelTime": 95,
            "confidence": 0.95,
            "roadName": "A100"
        }
    });

    Mock::given(method("GET"))
        .and(path("/traffic/services/4/flowSegmentData/absolute/10/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let traffic = TomTomTraffic::with_base_url(client, server.uri());

    let coordinate = Coordinate::new(52.52, 13.405).unwrap();
    let options = TrafficOptions::default();
    let response = traffic.get_traffic(&coordinate, &options).await.unwrap();

    assert_eq!(response.flows.len(), 1);
    assert_eq!(response.flows[0].speed, Some(65.0));
    assert_eq!(response.flows[0].free_flow_speed, Some(80.0));
    assert_eq!(response.flows[0].road_name, Some("A100".to_string()));
}

#[tokio::test]
async fn test_traffic_incidents_contract() {
    let server = MockServer::start().await;

    // Mock the flow endpoint
    let flow_response = serde_json::json!({
        "flowSegmentData": {
            "frc": "FRC0",
            "currentSpeed": 30,
            "freeFlowSpeed": 80,
            "confidence": 0.9,
            "roadName": "A100"
        }
    });

    Mock::given(method("GET"))
        .and(path("/traffic/services/4/flowSegmentData/absolute/10/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flow_response))
        .mount(&server)
        .await;

    // Mock the incidents endpoint
    let incidents_response = serde_json::json!({
        "incidents": [
            {
                "id": "inc123",
                "type": "Accident",
                "severity": "major",
                "description": "Multi-vehicle accident on A100",
                "from": "A100 North",
                "to": "A100 South",
                "delay": 600,
                "length": 2.5,
                "startTime": "2026-04-12T08:00:00Z",
                "endTime": "2026-04-12T12:00:00Z"
            },
            {
                "id": "inc124",
                "type": "Construction",
                "severity": "moderate",
                "description": "Roadworks on A100",
                "startTime": "2026-04-12T09:00:00Z",
                "endTime": "2026-04-12T17:00:00Z"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/traffic/services/5/incidentDetails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(incidents_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let traffic = TomTomTraffic::with_base_url(client, server.uri());

    let coordinate = Coordinate::new(52.52, 13.405).unwrap();
    let options = TrafficOptions {
        include_incidents: Some(true),
        radius: Some(5000.0),
        ..Default::default()
    };
    let response = traffic.get_traffic(&coordinate, &options).await.unwrap();

    assert_eq!(response.flows.len(), 1);
    assert_eq!(response.flows[0].speed, Some(30.0));
    assert_eq!(response.incidents.len(), 2);
    assert_eq!(response.incidents[0].id, Some("inc123".to_string()));
    assert_eq!(
        response.incidents[0].severity,
        Some(IncidentSeverity::Major)
    );
    assert_eq!(
        response.incidents[1].severity,
        Some(IncidentSeverity::Moderate)
    );
    assert_eq!(
        response.incidents[0].description,
        Some("Multi-vehicle accident on A100".to_string())
    );
}

/// A caller-supplied box drives the incident request in TomTom's
/// `minLon,minLat,maxLon,maxLat` order and wins over a radius.
#[tokio::test]
async fn a_bounding_box_drives_the_incident_query_in_tomtom_order() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/traffic/services/4/flowSegmentData/absolute/10/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "flowSegmentData": {
                "frc": "FRC0",
                "currentSpeed": 30,
                "freeFlowSpeed": 80,
                "confidence": 0.9,
                "roadName": "A100"
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/traffic/services/5/incidentDetails"))
        .and(query_param("bbox", "13.3,52.4,13.5,52.6"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "incidents": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let traffic = TomTomTraffic::with_base_url(client, server.uri());
    let options = TrafficOptions {
        bounding_box: Some(BoundingBox::new(
            Coordinate::new(52.6, 13.5).unwrap(),
            Coordinate::new(52.4, 13.3).unwrap(),
        )),
        // A radius beside a box must not win over the caller's viewport.
        radius: Some(5_000.0),
        include_incidents: Some(true),
        ..TrafficOptions::default()
    };

    let response = traffic
        .get_traffic(&Coordinate::new(52.52, 13.405).unwrap(), &options)
        .await
        .unwrap();
    server.verify().await;
    assert_eq!(response.flows.len(), 1);
    assert!(response.incidents.is_empty());
}
