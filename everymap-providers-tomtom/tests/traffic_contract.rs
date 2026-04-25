use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::traffic::{IncidentSeverity, TrafficOptions, TrafficProvider};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomTraffic;
use std::sync::Arc;
use wiremock::matchers::{method, path};
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

    let coord = Coordinate::new(52.52, 13.405).unwrap();
    let opts = TrafficOptions::default();
    let res = traffic.get_traffic(&coord, &opts).await.unwrap();

    assert_eq!(res.flows.len(), 1);
    assert_eq!(res.flows[0].speed, Some(65.0));
    assert_eq!(res.flows[0].free_flow_speed, Some(80.0));
    assert_eq!(res.flows[0].road_name, Some("A100".to_string()));
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

    let coord = Coordinate::new(52.52, 13.405).unwrap();
    let opts = TrafficOptions {
        include_incidents: Some(true),
        radius: Some(5000.0),
        ..Default::default()
    };
    let res = traffic.get_traffic(&coord, &opts).await.unwrap();

    assert_eq!(res.flows.len(), 1);
    assert_eq!(res.flows[0].speed, Some(30.0));
    assert_eq!(res.incidents.len(), 1);
    assert_eq!(res.incidents[0].id, Some("inc123".to_string()));
    assert_eq!(res.incidents[0].severity, Some(IncidentSeverity::Major));
    assert_eq!(
        res.incidents[0].description,
        Some("Multi-vehicle accident on A100".to_string())
    );
}
