use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::domains::routing::DepartureTime;
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxRouteMatcher;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "matchings": [
            {
                "confidence": 0.95,
                "distance": 1500.0,
                "duration": 120.0,
                "geometry": "~bidF_olxI~d~C",
                "legs": []
            }
        ],
        "tracepoints": [
            {
                "waypoint_index": 0,
                "location": [13.405, 52.52],
                "name": "Unter den Linden",
                "distance": 5.0
            },
            {
                "waypoint_index": 1,
                "location": [13.45, 52.55],
                "name": "Alexanderplatz",
                "distance": 3.0
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/matching/v5/mapbox/driving/13.405,52.52;13.45,52.55.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let matcher = MapBoxRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.55, 13.45).unwrap(),
    ];
    let options = MatchingOptions::default();

    let response = matcher.match_route(&points, &options).await.unwrap();

    assert_eq!(response.matched_points.len(), 2);
    assert_eq!(response.distance, 1500.0);
    assert_eq!(
        response.matched_points[0].road_name,
        Some("Unter den Linden".to_string())
    );
}

#[tokio::test]
async fn test_matching_with_heading() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "matchings": [{
            "confidence": 0.95,
            "distance": 1500.0,
            "duration": 120.0,
            "geometry": "~bidF_olxI~d~C",
            "legs": []
        }],
        "tracepoints": [
            { "waypoint_index": 0, "location": [13.405, 52.52], "name": "Point 1", "distance": 5.0 },
            { "waypoint_index": 1, "location": [13.45, 52.55], "name": "Point 2", "distance": 3.0 }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/matching/v5/mapbox/driving/13.405,52.52;13.45,52.55.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let matcher = MapBoxRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.55, 13.45).unwrap(),
    ];
    let options = MatchingOptions {
        heading: Some(90.0),
        ..Default::default()
    };

    let response = matcher.match_route(&points, &options).await.unwrap();
    assert_eq!(response.matched_points.len(), 2);
}

#[tokio::test]
async fn test_matching_with_departure_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "matchings": [{
            "confidence": 0.92,
            "distance": 1500.0,
            "duration": 110.0,
            "geometry": "~bidF_olxI~d~C",
            "legs": []
        }],
        "tracepoints": [
            { "waypoint_index": 0, "location": [13.405, 52.52], "name": "Berlin", "distance": 5.0 },
            { "waypoint_index": 1, "location": [13.45, 52.55], "name": "Alexanderplatz", "distance": 3.0 }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/matching/v5/mapbox/driving/13.405,52.52;13.45,52.55.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let matcher = MapBoxRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.55, 13.45).unwrap(),
    ];
    let options = MatchingOptions {
        departure_time: Some(DepartureTime::Now),
        ..Default::default()
    };

    let response = matcher.match_route(&points, &options).await.unwrap();
    assert_eq!(response.matched_points.len(), 2);
}
