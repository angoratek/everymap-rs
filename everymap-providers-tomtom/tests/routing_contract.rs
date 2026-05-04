use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
use everymap_core::types::Coordinate;
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::TomTomRouter;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "formatVersion": "0.0.12",
        "routes": [
            {
                "summary": {
                    "lengthInMeters": 947000,
                    "travelTimeInSeconds": 32400
                },
                "legs": [
                    {
                        "summary": {
                            "lengthInMeters": 947000,
                            "travelTimeInSeconds": 32400
                        },
                        "points": [
                            { "latitude": 52.52, "longitude": 13.405 },
                            { "latitude": 51.50, "longitude": -0.12 }
                        ]
                    }
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path(
            "/routing/1/calculateRoute/52.52,13.405:51.5,-0.12/json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(TomTomClient::new(auth));
    let router = TomTomRouter::with_base_url(client, server.uri());

    let start = Coordinate::new(52.52, 13.405).unwrap();
    let end = Coordinate::new(51.5, -0.12).unwrap();
    let options = RouteOptions {
        transport_mode: Some(TransportMode::Car),
        ..Default::default()
    };

    let response = router.calculate_route(&start, &end, &options).await.unwrap();

    assert_eq!(response.routes.len(), 1);
    assert_eq!(response.routes[0].distance, 947000.0);
    assert_eq!(response.routes[0].duration, 32400.0);
    assert_eq!(response.routes[0].geometry.points.len(), 2);
}
