use everymap_core::auth::{AuthProvider, OAuth2Provider};
use reqwest::header::AUTHORIZATION;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider_for(server: &MockServer, scope: Option<String>) -> OAuth2Provider {
    OAuth2Provider::new(
        format!("{}/token", server.uri()),
        "test-client-id".to_string(),
        "test-client-secret".to_string(),
        scope,
    )
}

fn sample_request() -> reqwest::RequestBuilder {
    reqwest::Client::new().get("http://localhost/unused")
}

#[tokio::test]
async fn test_apply_fetches_token_and_sets_bearer_header() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string_contains("grant_type=client_credentials"))
        .and(body_string_contains("client_id=test-client-id"))
        .and(body_string_contains("client_secret=test-client-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "token-abc",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    let builder = provider.apply(sample_request()).await.unwrap();
    let request = builder.build().unwrap();

    assert_eq!(
        request.headers().get(AUTHORIZATION).unwrap(),
        "Bearer token-abc"
    );

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
}

#[tokio::test]
async fn test_apply_sends_scope_parameter() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("scope=maps.read+routes.write"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "token-abc",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, Some("maps.read routes.write".to_string()));
    let builder = provider.apply(sample_request()).await.unwrap();
    let request = builder.build().unwrap();

    assert_eq!(
        request.headers().get(AUTHORIZATION).unwrap(),
        "Bearer token-abc"
    );
}

#[tokio::test]
async fn test_cached_token_is_reused_within_lifetime() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "token-abc",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    for _ in 0..3 {
        let builder = provider.apply(sample_request()).await.unwrap();
        let request = builder.build().unwrap();
        assert_eq!(
            request.headers().get(AUTHORIZATION).unwrap(),
            "Bearer token-abc"
        );
    }

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1, "token must be fetched exactly once");
}

#[tokio::test]
async fn test_expired_token_triggers_refresh() {
    let server = MockServer::start().await;

    // expires_in=0 with the 30s safety skew means the token is immediately
    // expired, so every apply() must fetch a fresh token.
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "token-abc",
            "expires_in": 0
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    for _ in 0..2 {
        let builder = provider.apply(sample_request()).await.unwrap();
        let request = builder.build().unwrap();
        assert_eq!(
            request.headers().get(AUTHORIZATION).unwrap(),
            "Bearer token-abc"
        );
    }

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 2, "expired token must trigger a refresh");
}

#[tokio::test]
async fn test_token_endpoint_error_maps_to_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_client"
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    let error = provider.apply(sample_request()).await.unwrap_err();

    assert!(error.is_auth_error());
    let message = format!("{}", error);
    assert!(message.contains("400"), "message: {}", message);
    assert!(message.contains("invalid_client"), "message: {}", message);
}

#[tokio::test]
async fn test_malformed_token_body_maps_to_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    let error = provider.apply(sample_request()).await.unwrap_err();

    assert!(error.is_auth_error());
    let message = format!("{}", error);
    assert!(
        message.contains("Failed to parse token response"),
        "message: {}",
        message
    );
}

#[tokio::test]
async fn test_missing_access_token_field_maps_to_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    let provider = provider_for(&server, None);
    let error = provider.apply(sample_request()).await.unwrap_err();

    assert!(error.is_auth_error());
    let message = format!("{}", error);
    assert!(
        message.contains("Failed to parse token response"),
        "message: {}",
        message
    );
}
