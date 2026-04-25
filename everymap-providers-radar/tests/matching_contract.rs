use everymap_core::auth::AuthProvider;
use everymap_core::domains::matching::MatchingOptions;
use everymap_core::domains::matching::RouteMatcher;
use everymap_core::types::Coordinate;
use everymap_providers_radar::{RadarClient, RadarRouteMatcher};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_matching_mock() -> (MockServer, RadarRouteMatcher) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> = Arc::new(everymap_core::auth::HeaderAuthProvider::new(
        "prj_test_pk_123".to_string(),
    ));
    let client = Arc::new(RadarClient::new(auth));
    let matcher =
        RadarRouteMatcher::with_base_url(client, format!("{}/v1/route/match", server.uri()));
    (server, matcher)
}

#[tokio::test]
async fn test_matching_contract() {
    let (server, matcher) = setup_matching_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/route/match"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "matchedPath": [
                { "latitude": 52.5163, "longitude": 13.3777, "originalIndex": 0 },
                { "latitude": 52.517, "longitude": 13.378, "originalIndex": 1 }
            ],
            "distance": { "value": 150, "text": "150m" },
            "geometry": { "polyline": "" }
        })))
        .mount(&server)
        .await;

    let points = vec![
        Coordinate::new(52.5163, 13.3777).unwrap(),
        Coordinate::new(52.517, 13.378).unwrap(),
    ];
    let result = matcher
        .match_route(&points, &MatchingOptions::default())
        .await;
    assert!(result.is_ok(), "match_route failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.matched_points.len(), 2);
    assert!((resp.distance - 150.0).abs() < 1.0);
}
