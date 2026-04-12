use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions};
use everymap_providers_google::GoogleAttributeProvider;
use everymap_providers_google::client::GoogleClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_attributes_by_place_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "ChIJxxxxxxxx",
                "speedLimit": 50.0,
                "units": "KPH"
            },
            {
                "placeId": "ChIJyyyyyyyy",
                "speedLimit": 30.0,
                "units": "KPH"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions {
        provider_extra: Some(serde_json::json!({
            "place_ids": ["ChIJxxxxxxxx", "ChIJyyyyyyyy"]
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await.unwrap();

    let limits = res.data.get("speedLimits").unwrap().as_array().unwrap();
    assert_eq!(limits.len(), 2);
    assert_eq!(limits[0]["speedLimit"], 50.0);
    assert_eq!(limits[1]["speedLimit"], 30.0);
}

#[tokio::test]
async fn test_attributes_along_path() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "road_segment_1",
                "speedLimit": 80.0,
                "units": "KPH"
            }
        ],
        "snappedPoints": [
            {
                "location": { "latitude": 52.52, "longitude": 13.405 },
                "originalIndex": 0,
                "placeId": "road_segment_1"
            },
            {
                "location": { "latitude": 52.53, "longitude": 13.41 },
                "originalIndex": 1,
                "placeId": "road_segment_1"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions {
        provider_extra: Some(serde_json::json!({
            "path": "52.52,13.405|52.53,13.41"
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await.unwrap();

    let limits = res.data.get("speedLimits").unwrap().as_array().unwrap();
    assert_eq!(limits.len(), 1);
    assert_eq!(limits[0]["speedLimit"], 80.0);
    let snapped = res.data.get("snappedPoints").unwrap().as_array().unwrap();
    assert_eq!(snapped.len(), 2);
}

#[tokio::test]
async fn test_attributes_missing_params() {
    let server = MockServer::start().await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions::default();

    let res = provider.get_attributes(&opts).await;
    assert!(res.is_err());
    let err_msg = format!("{}", res.unwrap_err());
    assert!(err_msg.contains("place_ids") || err_msg.contains("path"));
}

#[tokio::test]
async fn test_attributes_with_units() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "road_mph",
                "speedLimit": 55.0,
                "units": "MPH"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    let opts = AttributeOptions {
        provider_extra: Some(serde_json::json!({
            "place_ids": ["road_mph"],
            "units": "MPH"
        })),
        ..Default::default()
    };

    let res = provider.get_attributes(&opts).await.unwrap();

    let limits = res.data.get("speedLimits").unwrap().as_array().unwrap();
    assert_eq!(limits[0]["units"], "MPH");
    assert_eq!(limits[0]["speedLimit"], 55.0);
}