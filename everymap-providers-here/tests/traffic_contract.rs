use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::traffic::{HereFlowOptions, HereTraffic};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_traffic_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "results": [
            {
                "location": {
                    "description": "A100",
                    "length": 1200.0,
                    "functionalClass": 3,
                    "shape": []
                },
                "currentFlow": {
                    "speed": 40.0,
                    "speedUncapped": 42.0,
                    "freeFlow": 80.0,
                    "jamFactor": 2.5,
                    "confidence": 0.9
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/flow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let traffic_provider = HereTraffic::with_base_url(client, server.uri());

    let coordinate = Coordinate::new(52.52, 13.405).unwrap();
    let options = TrafficOptions::default();

    let response = traffic_provider.get_traffic(&coordinate, &options).await.unwrap();

    assert_eq!(response.flows.len(), 1);
    assert_eq!(response.flows[0].jam_factor.unwrap(), 2.5);
    assert!(response.incidents.is_empty());
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

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let traffic_provider = HereTraffic::with_base_url(client, server.uri());

    let response = traffic_provider
        .get_flow(
            Coordinate::new(52.52, 13.405).unwrap(),
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

    let mock_response = serde_json::json!({
        "results": [
            {
                "location": {
                    "description": "A1 Northbound",
                    "length": 500.0
                },
                "incident": {
                    "id": "INC_123",
                    "originalId": "orig_456",
                    "type": "accident",
                    "criticality": "major",
                    "status": "active",
                    "startTime": "2024-01-01T08:00:00Z",
                    "endTime": "2024-01-01T12:00:00Z",
                    "description": {
                        "value": "Major accident on A1",
                        "language": "en"
                    }
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/incidents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let traffic_provider = HereTraffic::with_base_url(client, server.uri());

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
