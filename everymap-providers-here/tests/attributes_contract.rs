use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions};
use everymap_providers_here::domain::attributes::HereAttributeProvider;
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_attributes_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[13.405, 52.520], [13.406, 52.521]]
                },
                "properties": {
                    "LINK_ID": "123456",
                    "FUNCTIONAL_CLASS": 3,
                    "SPEED_LIMIT": 50
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/roads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions {
        bbox: Some("52.5,13.4;52.6,13.5".to_string()),
        provider_extra: Some(serde_json::json!({
            "layer": "roads"
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await.unwrap();

    assert!(res.data.as_object().unwrap().contains_key("features"));
}

#[tokio::test]
async fn test_attributes_with_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [13.405, 52.520]
                },
                "properties": {
                    "LINK_ID": "789012",
                    "ROAD_CLASS": "A"
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/attributes/roads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let provider = HereAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions {
        provider_extra: Some(serde_json::json!({
            "layer": "roads",
            "ids": ["789012"]
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await.unwrap();
    assert!(res.data.as_object().unwrap().contains_key("features"));
}