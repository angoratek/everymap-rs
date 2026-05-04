use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::matching::HereRouteMatcher;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "response": {
            "route": [{
                "waypoint": [
                    {
                        "mappedPosition": { "latitude": 52.5201, "longitude": 13.4051 },
                        "originalPosition": { "latitude": 52.52, "longitude": 13.405 },
                        "confidenceValue": 0.9
                    },
                    {
                        "mappedPosition": { "latitude": 52.5301, "longitude": 13.4101 },
                        "originalPosition": { "latitude": 52.53, "longitude": 13.41 },
                        "confidenceValue": 1.0
                    }
                ],
                "leg": [
                    { "length": 120.0, "travelTime": 15.0 }
                ]
            }]
        }
    });

    Mock::given(method("GET"))
        .and(path("/match/routelinks"))
        .and(query_param("waypoint0", "52.52,13.405"))
        .and(query_param("waypoint1", "52.53,13.41"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let matcher = HereRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let options = MatchingOptions::default();

    let response = matcher.match_route(&points, &options).await.unwrap();

    assert_eq!(response.distance, 120.0);
    assert_eq!(response.matched_points.len(), 2);
    assert_eq!(
        response.matched_points[0].coordinate,
        Coordinate::new(52.5201, 13.4051).unwrap()
    );
    assert_eq!(response.matched_points[0].confidence, Some(0.9));
}

#[tokio::test]
async fn test_matching_with_options() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "response": {
            "route": [{
                "waypoint": [
                    {
                        "mappedPosition": { "latitude": 52.5201, "longitude": 13.4051 },
                        "originalPosition": { "latitude": 52.52, "longitude": 13.405 },
                        "confidenceValue": 0.9
                    },
                    {
                        "mappedPosition": { "latitude": 52.5301, "longitude": 13.4101 },
                        "originalPosition": { "latitude": 52.53, "longitude": 13.41 },
                        "confidenceValue": 1.0
                    }
                ],
                "leg": [
                    { "length": 250.0, "travelTime": 30.0 }
                ]
            }]
        }
    });

    Mock::given(method("GET"))
        .and(path("/match/routelinks"))
        .and(query_param("waypoint0", "52.52,13.405"))
        .and(query_param("waypoint1", "52.53,13.41"))
        .and(query_param("mode", "fastest;car;traffic:disabled"))
        .and(query_param("mapMatchRadius", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let matcher = HereRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let options = MatchingOptions {
        provider_extra: Some(serde_json::json!({
            "mode": "car",
            "map_match_radius": 50
        })),
        ..Default::default()
    };

    let response = matcher.match_route(&points, &options).await.unwrap();

    assert_eq!(response.distance, 250.0);
    assert_eq!(response.matched_points.len(), 2);
}
