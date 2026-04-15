use everymap_core::domains::tour::TourPlanner;
use everymap_core::domains::tour::TourOptions;
use everymap_core::auth::AuthProvider;
use everymap_core::types::Coordinate;
use everymap_providers_radar::{RadarTourPlanner, RadarClient};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::sync::Arc;

async fn setup_tour_mock() -> (MockServer, RadarTourPlanner) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> = Arc::new(everymap_core::auth::HeaderAuthProvider::new("prj_test_pk_123".to_string()));
    let client = Arc::new(RadarClient::new(auth));
    let planner = RadarTourPlanner::with_base_url(
        client,
        format!("{}/v1/route/optimize", server.uri()),
    );
    (server, planner)
}

#[tokio::test]
async fn test_tour_contract() {
    let (server, planner) = setup_tour_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/route/optimize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "route": {
                "distance": { "value": 15000, "text": "15 km" },
                "duration": { "value": 25, "text": "25 min" },
                "legs": [
                    {
                        "startLocation": { "latitude": 52.5163, "longitude": 13.3777 },
                        "endLocation": { "latitude": 52.52, "longitude": 13.405 },
                        "startIndex": 0,
                        "endIndex": 1,
                        "distance": { "value": 8000, "text": "8 km" },
                        "duration": { "value": 12, "text": "12 min" }
                    },
                    {
                        "startLocation": { "latitude": 52.52, "longitude": 13.405 },
                        "endLocation": { "latitude": 52.53, "longitude": 13.41 },
                        "startIndex": 1,
                        "endIndex": 2,
                        "distance": { "value": 7000, "text": "7 km" },
                        "duration": { "value": 13, "text": "13 min" }
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let stops = vec![
        Coordinate::new(52.5163, 13.3777).unwrap(),
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let result = planner.optimize_tour(&stops, &TourOptions::default()).await.unwrap();
    assert_eq!(result.stops.len(), 3); // 2 legs start + 1 final end
    assert!((result.total_distance.unwrap() - 15000.0).abs() < 1.0);
}