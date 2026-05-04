use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GooglePositioner;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_positioning_contract() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 52.5200,
            "lng": 13.4050
        },
        "accuracy": 50.0
    });

    Mock::given(method("POST"))
        .and(path("/geolocate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let positioner = GooglePositioner::with_base_url(client, server.uri());

    let options = PositioningOptions::default();

    let response = positioner.get_position(&options).await.unwrap();

    assert_eq!(response.coordinate.lat, 52.5200);
    assert_eq!(response.coordinate.lng, 13.4050);
    assert_eq!(response.accuracy, Some(50.0));
    assert!(response.altitude.is_none());
}

#[tokio::test]
async fn test_positioning_with_wifi() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 37.4220,
            "lng": -122.0841
        },
        "accuracy": 25.0
    });

    Mock::given(method("POST"))
        .and(path("/geolocate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let positioner = GooglePositioner::with_base_url(client, server.uri());

    let options = PositioningOptions {
        provider_extra: Some(serde_json::json!({
            "considerIp": true,
            "wifiAccessPoints": [
                {
                    "macAddress": "00:1A:2B:3C:4D:5E",
                    "signalStrength": -65,
                    "channel": 6
                }
            ]
        })),
    };

    let response = positioner.get_position(&options).await.unwrap();

    assert_eq!(response.coordinate.lat, 37.4220);
    assert_eq!(response.coordinate.lng, -122.0841);
    assert_eq!(response.accuracy, Some(25.0));
}

#[tokio::test]
async fn test_positioning_locate_rich() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "location": {
            "lat": 51.5074,
            "lng": -0.1278
        },
        "accuracy": 100.0
    });

    Mock::given(method("POST"))
        .and(path("/geolocate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new(
        "test-key".to_string(),
        "key".to_string(),
    ));
    let client = Arc::new(GoogleClient::new(auth));
    let positioner = GooglePositioner::with_base_url(client, server.uri());

    use everymap_providers_google::GooglePositioningOptions;
    let google_opts = GooglePositioningOptions {
        consider_ip: Some(true),
        wifi_access_points: Some(vec![everymap_providers_google::GoogleWifiAccessPoint {
            mac_address: Some("00:11:22:33:44:55".to_string()),
            signal_strength: Some(-70),
            age: None,
            channel: Some(11),
            signal_to_noise_ratio: None,
        }]),
        cell_towers: None,
    };

    let result = positioner.locate(google_opts).await.unwrap();

    assert_eq!(result.location.lat, 51.5074);
    assert_eq!(result.location.lng, -0.1278);
    assert_eq!(result.accuracy, 100.0);
}
