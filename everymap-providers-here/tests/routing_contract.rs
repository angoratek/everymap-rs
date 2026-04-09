use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::types::Coordinate;
use everymap_core::domains::routing::{RouteRequest, Router};
use everymap_providers_here::domain::routing::{HereRouter, HereRouteOptions, TransportMode};
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_routing_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "routes": [
            {
                "sections": [
                    {
                        "summary": { "length": 1500.0, "duration": 300.0 },
                        "polyline": { "polyline": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
                    }
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/routes"))
        .and(query_param("transportMode", "car"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let router = HereRouter::with_base_url(client, server.uri());

    let req = RouteRequest {
        start: Coordinate::new(52.52, 13.405).unwrap(),
        end: Coordinate::new(52.53, 13.41).unwrap(),
        options: HereRouteOptions {
            transport_mode: TransportMode::Car,
            ..Default::default()
        },
    };

    let res = router.calculate_route(req).await.unwrap();

    assert_eq!(res.routes[0].distance, 1500.0);
    assert_eq!(res.routes[0].duration, 300.0);
}