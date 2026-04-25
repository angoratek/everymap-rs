use everymap_core::auth::AuthProvider;
use everymap_core::domains::routing::RouteOptions;
use everymap_core::domains::routing::Router;
use everymap_core::types::Coordinate;
use everymap_providers_radar::{RadarClient, RadarRouter};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_router_mock() -> (MockServer, RadarRouter) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> = Arc::new(everymap_core::auth::HeaderAuthProvider::new(
        "prj_test_pk_123".to_string(),
    ));
    let client = Arc::new(RadarClient::new(auth));
    let router =
        RadarRouter::with_base_url(client, format!("{}/v1/route/directions", server.uri()));
    (server, router)
}

#[tokio::test]
async fn test_routing_contract() {
    let (server, router) = setup_router_mock().await;

    Mock::given(method("GET"))
        .and(path("/v1/route/directions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "routes": [{
                "duration": { "value": 120.5, "text": "2h 0m" },
                "distance": { "value": 250000, "text": "250 km" },
                "legs": [{
                    "startLocation": { "latitude": 52.5163, "longitude": 13.3777 },
                    "endLocation": { "latitude": 48.8566, "longitude": 2.3522 },
                    "duration": { "value": 120.5, "text": "2h 0m" },
                    "distance": { "value": 250000, "text": "250 km" },
                    "steps": [{
                        "distance": { "value": 500, "text": "0.5 km" },
                        "duration": { "value": 1.0, "text": "1 min" },
                        "start_location": { "latitude": 52.5163, "longitude": 13.3777 },
                        "end_location": { "latitude": 52.52, "longitude": 13.38 },
                        "bearing_before": 0,
                        "bearing_after": 90,
                        "instructions": "Head north on Pariser Platz",
                        "maneuver": "start"
                    }]
                }],
                "geometry": { "polyline": "" }
            }]
        })))
        .mount(&server)
        .await;

    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let result = router
        .calculate_route(&start, &end, &RouteOptions::default())
        .await
        .unwrap();
    assert_eq!(result.routes.len(), 1);
    let route = &result.routes[0];
    assert!((route.distance - 250000.0).abs() < 1.0);
    assert!((route.duration - 120.5 * 60.0).abs() < 1.0); // converted from minutes to seconds
    assert_eq!(route.steps.len(), 1);
}
