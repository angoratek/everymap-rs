use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
use everymap_core::types::Coordinate;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleRouter;
use std::sync::Arc;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    // Encoded polyline for a simple route
    let mock_response = serde_json::json!({
        "routes": [
            {
                "summary": "A2/E50",
                "legs": [
                    {
                        "distance": { "value": 1052000, "text": "1,052 km" },
                        "duration": { "value": 36720, "text": "about 10 hours" },
                        "startLocation": { "lat": 52.5163, "lng": 13.3777 },
                        "endLocation": { "lat": 48.8566, "lng": 2.3522 },
                        "startAddress": "Berlin, Germany",
                        "endAddress": "Paris, France",
                        "steps": [
                            {
                                "distance": { "value": 500, "text": "0.5 km" },
                                "duration": { "value": 60, "text": "1 min" },
                                "startLocation": { "lat": 52.5163, "lng": 13.3777 },
                                "endLocation": { "lat": 52.5170, "lng": 13.3780 },
                                "htmlInstructions": "Head <b>north</b>",
                                "maneuver": "turn-left",
                                "polyline": { "points": "_m_I??_o~@" },
                                "travelMode": "DRIVING"
                            },
                            {
                                "distance": { "value": 1051500, "text": "1,051.5 km" },
                                "duration": { "value": 36660, "text": "about 10 hours" },
                                "startLocation": { "lat": 52.5170, "lng": 13.3780 },
                                "endLocation": { "lat": 48.8566, "lng": 2.3522 },
                                "htmlInstructions": "Continue to <b>Paris</b>",
                                "polyline": { "points": "c~_I??~o~@" },
                                "travelMode": "DRIVING"
                            }
                        ]
                    }
                ],
                "overviewPolyline": { "points": "_m_I??c~_I??~o~@" },
                "bounds": {
                    "northeast": { "lat": 52.5170, "lng": 13.3780 },
                    "southwest": { "lat": 48.8566, "lng": 2.3522 }
                },
                "copyrights": "Map data 2024",
                "warnings": [],
                "waypointOrder": []
            }
        ],
        "status": "OK",
        "geocodedWaypoints": [
            { "geocoderStatus": "OK", "placeId": "ChIJAYWNSLSvqEcRkR7QsRVEs", "types": ["locality", "political"] },
            { "geocoderStatus": "OK", "placeId": "ChIJD7fiBh9u5kcRYJSMa1j2v", "types": ["locality", "political"] }
        ]
    });

    // GoogleRouter uses base_url directly as the full URL, so with_base_url
    // replaces the entire URL with the mock server URI (path becomes "/")
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let router = GoogleRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let opts = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &opts).await.unwrap();
    assert_eq!(result.routes.len(), 1);

    let route = &result.routes[0];
    assert_eq!(route.distance, 1052000.0);
    assert_eq!(route.duration, 36720.0);
    assert!(!route.geometry.points.is_empty());
    assert_eq!(route.steps.len(), 2);
    assert_eq!(
        route.steps[0].instruction,
        Some("Head <b>north</b>".to_string())
    );
    assert!(route.bounding_box.is_some());
}

#[tokio::test]
async fn test_routing_with_pedestrian_mode() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "summary": "Walking route",
                "legs": [
                    {
                        "distance": { "value": 800000, "text": "800 km" },
                        "duration": { "value": 576000, "text": "about 6.7 days" },
                        "startLocation": { "lat": 52.5163, "lng": 13.3777 },
                        "endLocation": { "lat": 48.8566, "lng": 2.3522 },
                        "startAddress": "Berlin, Germany",
                        "endAddress": "Paris, France",
                        "steps": []
                    }
                ],
                "overviewPolyline": { "points": "_m_I??~o~@" },
                "bounds": {
                    "northeast": { "lat": 52.52, "lng": 13.38 },
                    "southwest": { "lat": 48.85, "lng": 2.35 }
                },
                "warnings": [],
                "waypointOrder": []
            }
        ],
        "status": "OK"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let router = GoogleRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let opts = RouteOptions {
        transport_mode: Some(TransportMode::Pedestrian),
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &opts).await.unwrap();
    assert_eq!(result.routes.len(), 1);
    assert_eq!(result.routes[0].distance, 800000.0);
    assert_eq!(result.routes[0].duration, 576000.0);
}

#[tokio::test]
async fn test_routing_error_response() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [],
        "status": "NOT_FOUND",
        "errorMessage": "Could not find route between origin and destination."
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let router = GoogleRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(48.856, 2.352).unwrap();
    let opts = RouteOptions::default();

    let result = router.calculate_route(&start, &end, &opts).await;
    assert!(result.is_err());
}
