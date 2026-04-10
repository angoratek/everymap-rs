use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_providers_here::domain::positioning::{
    HerePositioner, HerePositioningOptions, WlanAccessPoint,
};
use everymap_providers_here::client::HereClient;
use everymap_core::auth::ApiKeyProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_positioning_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 52.5201,
            "lng": 13.4051,
            "accuracy": 50.0
        },
        "altitude": {
            "value": 34.0,
            "accuracy": 10.0
        }
    });

    Mock::given(method("POST"))
        .and(path("/position"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let positioner = HerePositioner::with_base_url(client, server.uri());

    let opts = PositioningOptions::default();

    let res = positioner.get_position(&opts).await.unwrap();

    assert_eq!(res.coordinate.lat, 52.5201);
    assert_eq!(res.coordinate.lng, 13.4051);
    assert_eq!(res.accuracy, Some(50.0));
    assert!(res.altitude.is_some());
    assert_eq!(res.altitude, Some(34.0));
    assert_eq!(res.altitude_accuracy, Some(10.0));
}

#[tokio::test]
async fn test_positioning_with_wlan() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 52.52,
            "lng": 13.41,
            "accuracy": 25.0
        }
    });

    Mock::given(method("POST"))
        .and(path("/position"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let positioner = HerePositioner::with_base_url(client, server.uri());

    let wlan_opts = HerePositioningOptions {
        wlan: Some(vec![WlanAccessPoint {
            mac: "00:11:22:33:44:55".to_string(),
            signal_strength: Some(-70),
            age: Some(5000),
            channel: Some(6),
            ssid: None,
            signal_to_noise_ratio: None,
        }]),
        cell: None,
        bluetooth: None,
        fallback: None,
    };
    let opts = PositioningOptions {
        provider_extra: Some(serde_json::to_value(wlan_opts).unwrap()),
    };

    let res = positioner.get_position(&opts).await.unwrap();

    assert_eq!(res.coordinate.lat, 52.52);
    assert_eq!(res.coordinate.lng, 13.41);
    assert_eq!(res.accuracy, Some(25.0));
    assert!(res.altitude.is_none());
}

#[tokio::test]
async fn test_positioning_locate() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 52.5201,
            "lng": 13.4051,
            "accuracy": 50.0
        },
        "altitude": {
            "value": 34.0,
            "accuracy": 10.0
        }
    });

    Mock::given(method("POST"))
        .and(path("/position"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));
    let positioner = HerePositioner::with_base_url(client, server.uri());

    let res = positioner.locate(HerePositioningOptions::default()).await.unwrap();

    assert_eq!(res.location.lat, 52.5201);
    assert_eq!(res.location.lng, 13.4051);
    assert_eq!(res.location.accuracy, Some(50.0));
    assert!(res.altitude.is_some());
    let alt = res.altitude.as_ref().unwrap();
    assert_eq!(alt.value, Some(34.0));
    assert_eq!(alt.accuracy, Some(10.0));
}