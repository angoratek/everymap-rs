use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::TomTomRouteMatcher;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "snappedPoints": [
            {
                "coordinate": { "latitude": 52.5201, "longitude": 13.4051 },
                "originalIndex": 0
            },
            {
                "coordinate": { "latitude": 52.5301, "longitude": 13.4101 },
                "originalIndex": 1
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/snapToRoads/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));
    let matcher = TomTomRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions::default();

    let res = matcher.match_route(&points, &opts).await.unwrap();

    assert_eq!(res.matched_points.len(), 2);
    assert_eq!(res.matched_points[0].coordinate.lat, 52.5201);
    assert_eq!(res.matched_points[0].coordinate.lng, 13.4051);
}