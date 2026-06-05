use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider, RangeType};
use everymap_core::domains::routing::AvoidType;
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomIsoline;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_isoline_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "formatVersion": "0.0.1",
        "reachableRange": {
            "center": { "latitude": 52.52, "longitude": 13.405 },
            "boundary": [
                { "latitude": 52.55, "longitude": 13.405 },
                { "latitude": 52.54, "longitude": 13.45 },
                { "latitude": 52.50, "longitude": 13.45 },
                { "latitude": 52.49, "longitude": 13.405 },
                { "latitude": 52.50, "longitude": 13.36 },
                { "latitude": 52.54, "longitude": 13.36 }
            ]
        },
        "report": { "effectiveSettings": {} }
    });

    Mock::given(method("GET"))
        .and(path("/routing/1/calculateReachableRange/52.52,13.405/json"))
        .and(query_param("distanceBudgetInMeters", "5000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let isoline = TomTomIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(RangeType::Distance),
        ..Default::default()
    };

    let response = isoline.get_isoline(&center, 5000.0, &options).await.unwrap();

    assert_eq!(response.isolines.len(), 1);
    assert_eq!(response.isolines[0].polygon.len(), 6);
    assert_eq!(response.isolines[0].range, Some(5000.0));
}

#[tokio::test]
async fn test_isoline_with_avoid_tunnels() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "reachableRange": {
            "center": { "latitude": 52.52, "longitude": 13.405 },
            "boundary": [
                { "latitude": 52.55, "longitude": 13.405 },
                { "latitude": 52.54, "longitude": 13.45 },
                { "latitude": 52.49, "longitude": 13.45 },
                { "latitude": 52.49, "longitude": 13.36 },
                { "latitude": 52.54, "longitude": 13.36 }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/routing/1/calculateReachableRange/52.52,13.405/json"))
        .and(query_param("distanceBudgetInMeters", "5000"))
        .and(query_param("avoid", "tunnels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));
    let isoline = TomTomIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(RangeType::Distance),
        avoid: vec![AvoidType::Tunnels],
        ..Default::default()
    };

    let response = isoline.get_isoline(&center, 5000.0, &options).await.unwrap();
    assert_eq!(response.isolines.len(), 1);
}

#[tokio::test]
async fn test_isoline_range_type_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "reachableRange": {
            "center": { "latitude": 52.52, "longitude": 13.405 },
            "boundary": [
                { "latitude": 52.53, "longitude": 13.405 },
                { "latitude": 52.52, "longitude": 13.42 },
                { "latitude": 52.50, "longitude": 13.405 },
                { "latitude": 52.52, "longitude": 13.39 }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/routing/1/calculateReachableRange/52.52,13.405/json"))
        .and(query_param("timeBudgetInSec", "600"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));
    let isoline = TomTomIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(RangeType::Time),
        ..Default::default()
    };

    let response = isoline.get_isoline(&center, 600.0, &options).await.unwrap();
    assert_eq!(response.isolines.len(), 1);
}
