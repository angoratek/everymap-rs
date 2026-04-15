use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions};
use everymap_core::auth::{AuthProvider, HeaderAuthProvider};
use everymap_providers_radar::{RadarFraudDetector, RadarClient};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::sync::Arc;

async fn setup_fraud_mock() -> (MockServer, RadarFraudDetector) {
    let server = MockServer::start().await;
    let auth: Arc<dyn AuthProvider> = Arc::new(HeaderAuthProvider::new("prj_test_pk_123".to_string()));
    let client = Arc::new(RadarClient::new(auth));
    let detector = RadarFraudDetector::with_base_url(
        client,
        format!("{}/v1/track", server.uri()),
    );
    (server, detector)
}

#[tokio::test]
async fn test_check_fraud_contract_clean() {
    let (server, detector) = setup_fraud_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/track"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "user": {
                "fraud": {
                    "verified": true,
                    "passed": true,
                    "mocked": false,
                    "jumped": false,
                    "compromised": false,
                    "inaccurate": false,
                    "proxy": false,
                    "sharing": false,
                    "blocked": false,
                    "bypassed": false
                },
                "userId": "user_1",
                "deviceId": "dev_1"
            },
            "events": []
        })))
        .mount(&server)
        .await;

    let opts = FraudCheckOptions {
        device_id: "dev_1".to_string(),
        latitude: 40.7128,
        longitude: -74.006,
        accuracy: 10.0,
        user_id: Some("user_1".to_string()),
        ..Default::default()
    };
    let result = detector.check_fraud(&opts).await.unwrap();
    assert!(result.verified);
    assert!(result.passed);
    assert!(!result.mocked);
    assert!(!result.jumped);
    assert!(!result.proxy);
    assert!(!result.blocked);
}

#[tokio::test]
async fn test_check_fraud_contract_spoofed() {
    let (server, detector) = setup_fraud_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/track"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "user": {
                "fraud": {
                    "verified": false,
                    "passed": false,
                    "mocked": true,
                    "jumped": true,
                    "compromised": false,
                    "inaccurate": true,
                    "proxy": true,
                    "sharing": false,
                    "blocked": true,
                    "bypassed": false
                },
                "userId": "user_bad",
                "deviceId": "dev_bad"
            },
            "events": []
        })))
        .mount(&server)
        .await;

    let opts = FraudCheckOptions {
        device_id: "dev_bad".to_string(),
        latitude: 40.7128,
        longitude: -74.006,
        accuracy: 10.0,
        ..Default::default()
    };
    let result = detector.check_fraud(&opts).await.unwrap();
    assert!(!result.verified);
    assert!(!result.passed);
    assert!(result.mocked);
    assert!(result.jumped);
    assert!(result.inaccurate);
    assert!(result.proxy);
    assert!(result.blocked);
}

#[tokio::test]
async fn test_check_fraud_contract_no_fraud_data() {
    let (server, detector) = setup_fraud_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/track"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 200 },
            "user": {
                "userId": "user_1",
                "deviceId": "dev_1"
            },
            "events": []
        })))
        .mount(&server)
        .await;

    let opts = FraudCheckOptions {
        device_id: "dev_1".to_string(),
        latitude: 40.7128,
        longitude: -74.006,
        accuracy: 10.0,
        ..Default::default()
    };
    let result = detector.check_fraud(&opts).await.unwrap();
    // When no fraud data, defaults should all be false
    assert!(!result.verified);
    assert!(!result.passed);
}

#[tokio::test]
async fn test_check_fraud_contract_error_response() {
    let (server, detector) = setup_fraud_mock().await;

    Mock::given(method("POST"))
        .and(path("/v1/track"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "meta": { "code": 401 },
            "user": {
                "userId": "",
                "deviceId": ""
            },
            "events": []
        })))
        .mount(&server)
        .await;

    let opts = FraudCheckOptions {
        device_id: "dev_1".to_string(),
        latitude: 40.7128,
        longitude: -74.006,
        accuracy: 10.0,
        ..Default::default()
    };
    let result = detector.check_fraud(&opts).await;
    assert!(result.is_err());
}