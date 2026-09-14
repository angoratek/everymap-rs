use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::tour::{TourOptions, TourPlanner};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxTourPlanner;
use std::sync::Arc;
use wiremock::matchers::{method, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_tour_optimization_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "trips": [
            {
                "distance": 25000.0,
                "duration": 1800.0,
                "geometry": "~bidF_olxI~d~C",
                "legs": []
            }
        ],
        "waypoints": [
            {
                "waypoint_index": 2,
                "trips_index": 0,
                "location": [13.405, 52.52],
                "name": "Berlin Center"
            },
            {
                "waypoint_index": 0,
                "trips_index": 0,
                "location": [13.35, 52.50],
                "name": "Berlin West"
            },
            {
                "waypoint_index": 1,
                "trips_index": 0,
                "location": [13.45, 52.55],
                "name": "Berlin East"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(query_param("overview", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let tour = MapBoxTourPlanner::with_base_url(client, server.uri());

    let stops = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.50, 13.35).unwrap(),
        Coordinate::new(52.55, 13.45).unwrap(),
    ];
    let options = TourOptions::default();

    let response = tour.optimize_tour(&stops, &options).await.unwrap();

    assert_eq!(response.stops.len(), 3);
    assert_eq!(response.total_distance, Some(25000.0));
    assert_eq!(response.total_duration, Some(1800.0));
}

#[tokio::test]
async fn test_tour_with_provider_extra_source_destination() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "trips": [{
            "distance": 12500.0,
            "duration": 900.0,
            "geometry": "~bidF_olxI~d~C",
            "legs": []
        }],
        "waypoints": [
            { "waypoint_index": 0, "trips_index": 0, "location": [13.405, 52.52], "name": "Start" }
        ]
    });

    Mock::given(method("GET"))
        .and(query_param("overview", "false"))
        .and(query_param("source", "first"))
        .and(query_param("destination", "last"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let tour = MapBoxTourPlanner::with_base_url(client, server.uri());

    let stops = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.50, 13.35).unwrap(),
    ];
    let options = TourOptions {
        provider_extra: Some(serde_json::json!({
            "source": "first",
            "destination": "last"
        })),
        ..Default::default()
    };

    let response = tour.optimize_tour(&stops, &options).await.unwrap();
    assert!(response.total_distance.is_some());
}
