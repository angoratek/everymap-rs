use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::tour::{TourPlanner, TourOptions};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::TomTomTourPlanner;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_tour_optimization_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "optimizedWaypoints": [
            {
                "providedIndex": 2,
                "point": { "latitude": 52.52, "longitude": 13.405 }
            },
            {
                "providedIndex": 0,
                "point": { "latitude": 52.50, "longitude": 13.35 }
            },
            {
                "providedIndex": 1,
                "point": { "latitude": 52.55, "longitude": 13.45 }
            }
        ],
        "summary": {
            "lengthInMeters": 12500.0,
            "travelTimeInSeconds": 900.0
        }
    });

    Mock::given(method("POST"))
        .and(path("/routing/waypointoptimization/1/api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));
    let tour = TomTomTourPlanner::with_base_url(client, server.uri());

    let stops = vec![
        Coordinate::new(52.50, 13.35).unwrap(),
        Coordinate::new(52.55, 13.45).unwrap(),
        Coordinate::new(52.52, 13.405).unwrap(),
    ];
    let opts = TourOptions::default();

    let res = tour.optimize_tour(&stops, &opts).await.unwrap();

    assert_eq!(res.stops.len(), 3);
    // First optimized waypoint should have providedIndex 2
    assert_eq!(res.stops[0].coordinate.lat, 52.52);
    assert_eq!(res.stops[0].coordinate.lng, 13.405);
    assert_eq!(res.total_distance, Some(12500.0));
    assert_eq!(res.total_duration, Some(900.0));
}