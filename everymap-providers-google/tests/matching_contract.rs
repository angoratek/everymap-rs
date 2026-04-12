use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::types::Coordinate;
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions};
use everymap_providers_google::GoogleRouteMatcher;
use everymap_providers_google::client::GoogleClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "snappedPoints": [
            {
                "location": { "latitude": 52.5201, "longitude": 13.4051 },
                "originalIndex": 0,
                "placeId": "ChIJxxxxxxxx"
            },
            {
                "location": { "latitude": 52.5301, "longitude": 13.4101 },
                "originalIndex": 1,
                "placeId": "ChIJyyyyyyyy"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/snapToRoads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let matcher = GoogleRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions::default();

    let res = matcher.match_route(&points, &opts).await.unwrap();

    assert_eq!(res.matched_points.len(), 2);
    assert_eq!(res.matched_points[0].coordinate, Coordinate::new(52.5201, 13.4051).unwrap());
    assert_eq!(res.matched_points[0].road_name, Some("ChIJxxxxxxxx".to_string()));
    assert_eq!(res.matched_points[1].coordinate, Coordinate::new(52.5301, 13.4101).unwrap());
    assert_eq!(res.distance, 0.0); // Google doesn't provide distance
}

#[tokio::test]
async fn test_matching_with_interpolate() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "snappedPoints": [
            {
                "location": { "latitude": 52.5201, "longitude": 13.4051 },
                "originalIndex": 0,
                "placeId": "road_1"
            },
            {
                "location": { "latitude": 52.5250, "longitude": 13.4075 },
                "placeId": "road_1"
            },
            {
                "location": { "latitude": 52.5301, "longitude": 13.4101 },
                "originalIndex": 1,
                "placeId": "road_2"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/snapToRoads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let matcher = GoogleRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions {
        provider_extra: Some(serde_json::json!({
            "interpolate": true
        })),
        ..Default::default()
    };

    let res = matcher.match_route(&points, &opts).await.unwrap();

    // 3 points: 2 original + 1 interpolated (no originalIndex)
    assert_eq!(res.matched_points.len(), 3);
    // Interpolated point has no original_index → road_name should still be present
    assert!(res.matched_points[1].road_name.is_some());
}