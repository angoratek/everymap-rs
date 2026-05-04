use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::isoline::RangeType as CoreRangeType;
use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider};
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::isoline::HereIsoline;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_isoline_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "isolines": [
            {
                "range": { "type": "time", "value": 1000 },
                "polyline": { "outer": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/isolines"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let isoline_provider = HereIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(CoreRangeType::Time),
        ..Default::default()
    };

    let response = isoline_provider
        .get_isoline(&center, 1000.0, &options)
        .await
        .unwrap();
    assert!(!response.isolines[0].polygon.is_empty());
}

#[tokio::test]
async fn test_isoline_with_routing_mode() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "isolines": [
            {
                "range": { "type": "distance", "value": 5000 },
                "polyline": { "outer": "BFoz5xJ67i1B1B7PzIhaxL7Y" }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/isolines"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "apiKey".to_string(),
    ));
    let client = Arc::new(HereClient::new(auth));
    let isoline_provider = HereIsoline::with_base_url(client, server.uri());

    let center = Coordinate::new(52.52, 13.405).unwrap();
    let options = IsolineOptions {
        range_type: Some(CoreRangeType::Distance),
        provider_extra: Some(serde_json::json!({
            "routing_mode": "short"
        })),
        ..Default::default()
    };

    let response = isoline_provider
        .get_isoline(&center, 5000.0, &options)
        .await
        .unwrap();
    assert!(!response.isolines[0].polygon.is_empty());
}
