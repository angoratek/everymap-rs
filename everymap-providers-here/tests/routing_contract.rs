use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{AvoidType, DepartureTime, RouteOptions, Router, TransportMode};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::routing::HereRouter;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "sections": [
                    {
                        "summary": { "length": 1500.0, "duration": 300.0 },
                        "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                    }
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();

    assert_eq!(response.routes[0].distance, 1500.0);
    assert_eq!(response.routes[0].duration, 300.0);
}

#[tokio::test]
async fn test_routing_with_avoid_tolls() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1600.0, "duration": 320.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();
    assert!(response.routes[0].distance > 0.0);
}

#[tokio::test]
async fn test_routing_with_alternatives() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "sections": [{
                    "summary": { "length": 1500.0, "duration": 300.0 },
                    "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                }]
            },
            {
                "sections": [{
                    "summary": { "length": 1600.0, "duration": 310.0 },
                    "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                }]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        alternatives: Some(2),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();
    assert_eq!(response.routes.len(), 2);
}

#[tokio::test]
async fn test_routing_with_departure_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1500.0, "duration": 280.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        departure_time: Some(DepartureTime::Timestamp(1715702400)),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();
    assert!(response.routes[0].duration > 0.0);
}

#[tokio::test]
async fn test_routing_with_language() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "sections": [{
                "summary": { "length": 1500.0, "duration": 300.0 },
                "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }]
        }]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(52.53, 13.41).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        language: Some("de-DE".to_string()),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();
    assert!(response.routes[0].distance > 0.0);
}
