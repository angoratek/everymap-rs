use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{DepartureTime, RouteOptions, Router};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::MapBoxRouter;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "routes": [
            {
                "distance": 947000.0,
                "duration": 32400.0,
                "geometry": "~bidF_olxI~d~C",
                "legs": [
                    {
                        "distance": 947000.0,
                        "duration": 32400.0,
                        "summary": "Berlin, Paris",
                        "steps": [
                            {
                                "distance": 500.0,
                                "duration": 60.0,
                                "instruction": "Head west",
                                "name": "Unter den Linden",
                                "maneuver": {
                                    "type": "depart",
                                    "location": [13.405, 52.52]
                                }
                            }
                        ]
                    }
                ]
            }
        ],
        "waypoints": [
            { "name": "Berlin", "location": [13.405, 52.52] },
            { "name": "Paris", "location": [2.3522, 48.8566] }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/directions/v5/mapbox/driving/13.405,52.52;2.3522,48.8566",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "pk.test123".to_string(),
        "access_token".to_string(),
    ));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions::default();

    let response = router.calculate_route(&start, &end, &options).await.unwrap();

    assert_eq!(response.routes.len(), 1);
    assert_eq!(response.routes[0].distance, 947000.0);
    assert_eq!(response.routes[0].duration, 32400.0);
}

#[tokio::test]
async fn test_routing_with_departure_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "code": "Ok",
        "routes": [{
            "distance": 947000.0,
            "duration": 31000.0,
            "geometry": "~bidF_olxI~d~C",
            "legs": [{
                "distance": 947000.0,
                "duration": 31000.0,
                "summary": "Berlin, Paris",
                "steps": []
            }]
        }],
        "waypoints": [
            { "name": "Berlin", "location": [13.405, 52.52] },
            { "name": "Paris", "location": [2.3522, 48.8566] }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/directions/v5/mapbox/driving/13.405,52.52;2.3522,48.8566"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("pk.test123".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));
    let router = MapBoxRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        departure_time: Some(DepartureTime::Now),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();
    assert_eq!(response.routes.len(), 1);
}
