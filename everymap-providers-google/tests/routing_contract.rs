use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{AvoidType, DepartureTime, RouteOptions, Router, TransportMode};
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
                        "start_location": { "lat": 52.5163, "lng": 13.3777 },
                        "end_location": { "lat": 48.8566, "lng": 2.3522 },
                        "start_address": "Berlin, Germany",
                        "end_address": "Paris, France",
                        "steps": [
                            {
                                "distance": { "value": 500, "text": "0.5 km" },
                                "duration": { "value": 60, "text": "1 min" },
                                "start_location": { "lat": 52.5163, "lng": 13.3777 },
                                "end_location": { "lat": 52.5170, "lng": 13.3780 },
                                "html_instructions": "Head <b>north</b>",
                                "maneuver": "turn-left",
                                "polyline": { "points": "_m_I??_o~@" },
                                "travel_mode": "DRIVING"
                            },
                            {
                                "distance": { "value": 1051500, "text": "1,051.5 km" },
                                "duration": { "value": 36660, "text": "about 10 hours" },
                                "start_location": { "lat": 52.5170, "lng": 13.3780 },
                                "end_location": { "lat": 48.8566, "lng": 2.3522 },
                                "html_instructions": "Continue to <b>Paris</b>",
                                "polyline": { "points": "c~_I??~o~@" },
                                "travel_mode": "DRIVING"
                            }
                        ]
                    }
                ],
                "overview_polyline": { "points": "_m_I??c~_I??~o~@" },
                "bounds": {
                    "northeast": { "lat": 52.5170, "lng": 13.3780 },
                    "southwest": { "lat": 48.8566, "lng": 2.3522 }
                },
                "copyrights": "Map data 2024",
                "warnings": [],
                "waypoint_order": []
            }
        ],
        "status": "OK",
        "geocoded_waypoints": [
            { "geocoder_status": "OK", "place_id": "ChIJAYWNSLSvqEcRkR7QsRVEs", "types": ["locality", "political"] },
            { "geocoder_status": "OK", "place_id": "ChIJD7fiBh9u5kcRYJSMa1j2v", "types": ["locality", "political"] }
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
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &options).await.unwrap();
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
                        "start_location": { "lat": 52.5163, "lng": 13.3777 },
                        "end_location": { "lat": 48.8566, "lng": 2.3522 },
                        "start_address": "Berlin, Germany",
                        "end_address": "Paris, France",
                        "steps": []
                    }
                ],
                "overview_polyline": { "points": "_m_I??~o~@" },
                "bounds": {
                    "northeast": { "lat": 52.52, "lng": 13.38 },
                    "southwest": { "lat": 48.85, "lng": 2.35 }
                },
                "warnings": [],
                "waypoint_order": []
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
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Pedestrian),
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &options).await.unwrap();
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
        "error_message": "Could not find route between origin and destination."
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
    let options = RouteOptions::default();

    let result = router.calculate_route(&start, &end, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_routing_with_avoid_tolls() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "summary": "A2",
            "legs": [{
                "distance": { "value": 950000, "text": "950 km" },
                "duration": { "value": 33000, "text": "about 9 hours" },
                "start_location": { "lat": 52.5163, "lng": 13.3777 },
                "end_location": { "lat": 48.8566, "lng": 2.3522 },
                "steps": []
            }],
            "overview_polyline": { "points": "_m_I??~o~@" },
            "bounds": {
                "northeast": { "lat": 52.5, "lng": 13.4 },
                "southwest": { "lat": 48.8, "lng": 2.3 }
            },
            "warnings": [],
            "waypoint_order": []
        }],
        "status": "OK"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let router = GoogleRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &options).await.unwrap();
    assert_eq!(result.routes.len(), 1);
}

#[tokio::test]
async fn test_routing_with_arrival_time() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [{
            "summary": "A2/E50",
            "legs": [{
                "distance": { "value": 1052000, "text": "1,052 km" },
                "duration": { "value": 36720, "text": "about 10 hours" },
                "start_location": { "lat": 52.5163, "lng": 13.3777 },
                "end_location": { "lat": 48.8566, "lng": 2.3522 },
                "steps": []
            }],
            "overview_polyline": { "points": "_m_I??~o~@" },
            "bounds": {
                "northeast": { "lat": 52.5, "lng": 13.4 },
                "southwest": { "lat": 48.8, "lng": 2.3 }
            },
            "warnings": [],
            "waypoint_order": []
        }],
        "status": "OK"
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let router = GoogleRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.5163, 13.3777).unwrap();
    let end = Coordinate::new(48.8566, 2.3522).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        arrival_time: Some(DepartureTime::Timestamp(1715702400)),
        ..Default::default()
    };

    let result = router.calculate_route(&start, &end, &options).await;
    // arrival_time will log a warning but should still function
    assert!(result.is_ok());
}
