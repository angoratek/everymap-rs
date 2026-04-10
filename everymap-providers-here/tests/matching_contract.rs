use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::types::Coordinate;
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions};
use everymap_providers_here::domain::matching::HereRouteMatcher;
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "trace": [
            { "lat": 52.5201, "lng": 13.4051 },
            { "lat": 52.5202, "lng": 13.4052 }
        ],
        "summary": { "length": 120.0 }
    });

    Mock::given(method("GET"))
        .and(path("/match/routelinks"))
        .and(query_param("trace", "52.52,13.405;52.53,13.41"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let matcher = HereRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions::default();

    let res = matcher.match_route(&points, &opts).await.unwrap();

    assert_eq!(res.distance, 120.0);
    assert_eq!(res.matched_points.len(), 2);
    assert_eq!(res.matched_points[0].coordinate, Coordinate::new(52.5201, 13.4051).unwrap());
}

#[tokio::test]
async fn test_matching_with_options() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "trace": [
            { "lat": 52.5201, "lng": 13.4051 },
            { "lat": 52.5202, "lng": 13.4052 }
        ],
        "summary": { "length": 250.0 }
    });

    Mock::given(method("GET"))
        .and(path("/match/routelinks"))
        .and(query_param("trace", "52.52,13.405;52.53,13.41"))
        .and(query_param("mode", "car"))
        .and(query_param("mapMatchRadius", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let matcher = HereRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions {
        provider_extra: Some(serde_json::json!({
            "mode": "car",
            "map_match_radius": 50
        })),
        ..Default::default()
    };

    let res = matcher.match_route(&points, &opts).await.unwrap();

    assert_eq!(res.distance, 250.0);
    assert_eq!(res.matched_points.len(), 2);
}