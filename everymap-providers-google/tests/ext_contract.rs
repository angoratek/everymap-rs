use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use everymap_core::auth::ApiKeyProvider;
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::ext::{GooglePositionerExt, GoogleAttributeExt};
use everymap_providers_google::GooglePositioner;
use everymap_providers_google::GoogleAttributeProvider;
use everymap_providers_google::GooglePositioningOptions;
use everymap_providers_google::GoogleWifiAccessPoint;
use std::sync::Arc;

// --- GooglePositionerExt tests ---

#[tokio::test]
async fn test_positioner_ext_locate() {
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

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let positioner = GooglePositioner::with_base_url(client, server.uri());

    // Call through extension trait with camelCase options
    let google_opts = GooglePositioningOptions {
        consider_ip: Some(true),
        wifi_access_points: Some(vec![GoogleWifiAccessPoint {
            mac_address: Some("00:11:22:33:44:55".to_string()),
            signal_strength: Some(-65),
            age: None,
            channel: Some(6),
            signal_to_noise_ratio: None,
        }]),
        cell_towers: None,
    };

    let result = GooglePositionerExt::locate(&positioner, google_opts).await.unwrap();

    assert_eq!(result.location.lat, 37.4220);
    assert_eq!(result.location.lng, -122.0841);
    assert_eq!(result.accuracy, 25.0);
}

// --- GoogleAttributeExt tests ---

#[tokio::test]
async fn test_attribute_ext_get_speed_limits_by_ids() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "ChIJext_1",
                "speedLimit": 50.0,
                "units": "KPH"
            },
            {
                "placeId": "ChIJext_2",
                "speedLimit": 80.0,
                "units": "KPH"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .and(query_param("placeId", "ChIJext_1"))
        .and(query_param("placeId", "ChIJext_2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let place_ids = vec!["ChIJext_1".to_string(), "ChIJext_2".to_string()];
    let res = GoogleAttributeExt::get_speed_limits_by_ids(&provider, &place_ids, None).await.unwrap();

    assert_eq!(res.speed_limits.len(), 2);
    assert_eq!(res.speed_limits[0].place_id.as_deref(), Some("ChIJext_1"));
    assert_eq!(res.speed_limits[0].speed_limit, Some(50.0));
    assert_eq!(res.speed_limits[1].place_id.as_deref(), Some("ChIJext_2"));
    assert_eq!(res.speed_limits[1].speed_limit, Some(80.0));
}

#[tokio::test]
async fn test_attribute_ext_get_speed_limits_by_ids_with_units() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "ChIJmph_1",
                "speedLimit": 55.0,
                "units": "MPH"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .and(query_param("placeId", "ChIJmph_1"))
        .and(query_param("units", "MPH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait with units
    let place_ids = vec!["ChIJmph_1".to_string()];
    let res = GoogleAttributeExt::get_speed_limits_by_ids(&provider, &place_ids, Some("MPH")).await.unwrap();

    assert_eq!(res.speed_limits.len(), 1);
    assert_eq!(res.speed_limits[0].units.as_deref(), Some("MPH"));
    assert_eq!(res.speed_limits[0].speed_limit, Some(55.0));
}

#[tokio::test]
async fn test_attribute_ext_get_speed_limits_along_path() {
    let server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "speedLimits": [
            {
                "placeId": "path_seg_1",
                "speedLimit": 100.0,
                "units": "KPH"
            }
        ],
        "snappedPoints": [
            {
                "location": { "latitude": 52.52, "longitude": 13.405 },
                "originalIndex": 0,
                "placeId": "path_seg_1"
            },
            {
                "location": { "latitude": 52.53, "longitude": 13.41 },
                "originalIndex": 1,
                "placeId": "path_seg_1"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/speedLimits"))
        .and(query_param("path", "52.52,13.405|52.53,13.41"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let auth = Arc::new(ApiKeyProvider::new("test-key".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));
    let provider = GoogleAttributeProvider::with_base_url(client, server.uri());

    // Call through extension trait
    let res = GoogleAttributeExt::get_speed_limits_along_path(
        &provider, "52.52,13.405|52.53,13.41", None,
    ).await.unwrap();

    assert_eq!(res.speed_limits.len(), 1);
    assert_eq!(res.speed_limits[0].speed_limit, Some(100.0));
    assert!(!res.snapped_points.is_empty());
    let snapped = &res.snapped_points;
    assert_eq!(snapped.len(), 2);
    assert_eq!(snapped[0].original_index, Some(0));
    assert_eq!(snapped[1].original_index, Some(1));
}