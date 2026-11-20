use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::domains::routing::{AvoidType, DepartureTime, TransportMode};
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleRouteMatcher;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
        .and(path("/v1/snapToRoads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let matcher = GoogleRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let options = MatchingOptions::default();

    let response = matcher.match_route(&points, &options).await.unwrap();

    assert_eq!(response.matched_points.len(), 2);
    assert_eq!(
        response.matched_points[0].coordinate,
        Coordinate::new(52.5201, 13.4051).unwrap()
    );
    assert_eq!(
        response.matched_points[0].road_name,
        Some("ChIJxxxxxxxx".to_string())
    );
    assert_eq!(
        response.matched_points[1].coordinate,
        Coordinate::new(52.5301, 13.4101).unwrap()
    );
    assert_eq!(response.distance, 0.0); // Google doesn't provide distance
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
        .and(path("/v1/snapToRoads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let matcher = GoogleRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let options = MatchingOptions {
        provider_extra: Some(serde_json::json!({
            "interpolate": true
        })),
        ..Default::default()
    };

    let response = matcher.match_route(&points, &options).await.unwrap();

    // 3 points: 2 original + 1 interpolated (no originalIndex)
    assert_eq!(response.matched_points.len(), 3);
    // Interpolated point has no original_index → road_name should still be present
    assert!(response.matched_points[1].road_name.is_some());
}

#[tokio::test]
async fn matching_sends_only_the_parameters_the_roads_api_defines() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "snappedPoints": [
            {
                "location": { "latitude": 52.5201, "longitude": 13.4051 },
                "placeId": "road_1"
            },
            {
                "location": { "latitude": 52.5301, "longitude": 13.4101 },
                "placeId": "road_2"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/v1/snapToRoads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let matcher = GoogleRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    // Every option the method cannot express, plus the reserved `snapping` key.
    let options = MatchingOptions {
        transport_mode: Some(TransportMode::Car),
        heading: Some(180.0),
        departure_time: Some(DepartureTime::Now),
        avoid: vec![AvoidType::Tolls],
        provider_extra: Some(serde_json::json!({
            "interpolate": true,
            "snapping": "anywhere"
        })),
    };

    matcher.match_route(&points, &options).await.unwrap();

    let requests = server
        .received_requests()
        .await
        .expect("wiremock records requests");
    let request = requests.last().expect("the matcher sends one request");
    let parameters: Vec<(String, String)> = request
        .url
        .query_pairs()
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();

    let sent = |name: &str| {
        parameters
            .iter()
            .any(|(parameter_name, _)| parameter_name == name)
    };

    assert!(sent("path"), "the points are sent as the path");
    assert!(
        parameters
            .iter()
            .any(|(name, value)| name == "interpolate" && value == "true"),
        "interpolate is carried through"
    );
    for unsupported in ["mode", "heading", "departure_time", "avoid", "snapping"] {
        assert!(
            !sent(unsupported),
            "{unsupported} has no Roads API counterpart and must not be sent"
        );
    }
}
