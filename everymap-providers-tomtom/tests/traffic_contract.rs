use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::TomTomTraffic;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
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