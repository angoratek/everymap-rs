use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::matching::{MatchingOptions, RouteMatcher};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomRouteMatcher;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_matching_contract() {
    let server = MockServer::start().await;

    // TomTom Snap to Roads returns GeoJSON-style projectedPoints
    let mock_response = serde_json::json!({
        "projectedPoints": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.4051, 52.5201]
                },
                "properties": {
                    "routeIndex": 0,
                    "snapResult": "Matched"
                }
            },
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.4101, 52.5301]
                },
                "properties": {
                    "routeIndex": 0,
                    "snapResult": "Matched"
                }
            }
        ],
        "route": [],
        "distances": {
            "total": 150.0,
            "ferry": 0,
            "road": 150.0,
            "privateRoad": 0,
            "publicRoad": 150.0,
            "offRoad": 0,
            "unit": "m"
        }
    });

    Mock::given(method("GET"))
        .and(path("/snapToRoads/1/snap"))
        .and(query_param("points", "13.405,52.52;13.41,52.53"))
        .and(query_param("fields", "{projectedPoints{type,geometry{type,coordinates},properties{routeIndex,snapResult}},route{type,geometry{type,coordinates}},distances{total,unit}}"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let matcher = TomTomRouteMatcher::with_base_url(client, server.uri());

    let points = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(52.53, 13.41).unwrap(),
    ];
    let opts = MatchingOptions::default();

    let res = matcher.match_route(&points, &opts).await.unwrap();

    assert_eq!(res.matched_points.len(), 2);
    // GeoJSON coordinates are [lon, lat], so lat=52.5201, lng=13.4051
    assert_eq!(res.matched_points[0].coordinate.lat, 52.5201);
    assert_eq!(res.matched_points[0].coordinate.lng, 13.4051);
    assert_eq!(res.distance, 150.0);
}
