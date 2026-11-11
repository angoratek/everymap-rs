//! Contract tests for the shared client's reliability features.

use everymap_core::auth::ApiKeyProvider;
use everymap_core::client::{
    ProviderClient, RequestObservation, RequestObserver, ResponseObservation, RetryPolicy,
    TokenBucket,
};
use everymap_core::http::{HttpRequest, Method};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn api_key_provider() -> Arc<ApiKeyProvider> {
    Arc::new(ApiKeyProvider::new(
        "super-secret".to_string(),
        "apiKey".to_string(),
    ))
}

fn client() -> ProviderClient {
    ProviderClient::new(api_key_provider(), "test")
}

async fn mount_status(server: &MockServer, status: u16) {
    mount_method_status(server, "GET", status).await;
}

async fn mount_method_status(server: &MockServer, http_method: &str, status: u16) {
    Mock::given(method(http_method))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(status))
        .mount(server)
        .await;
}

#[tokio::test]
async fn get_retries_server_errors_up_to_the_policy_limit() {
    let server = MockServer::start().await;
    mount_status(&server, 503).await;

    let client = client();
    let request = client.build_request(Method::Get, &format!("{}/resource", server.uri()));
    let error = client.request(request).await.unwrap_err();

    assert!(error.is_status(503));
    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 3, "default policy allows three attempts");
}

#[tokio::test]
async fn post_is_never_retried() {
    let server = MockServer::start().await;
    mount_method_status(&server, "POST", 503).await;

    let client = client();
    let request = client.build_request(Method::Post, &format!("{}/resource", server.uri()));
    let error = client.request(request).await.unwrap_err();

    assert!(error.is_status(503));
    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1, "non-idempotent requests are sent once");
}

#[tokio::test]
async fn long_retry_after_is_returned_instead_of_slept() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "60"))
        .mount(&server)
        .await;

    let client = client();
    let request = client.build_request(Method::Get, &format!("{}/resource", server.uri()));
    let start = Instant::now();
    let error = client.request(request).await.unwrap_err();

    assert!(error.is_rate_limited());
    assert!(start.elapsed() < Duration::from_secs(5));
    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
}

#[tokio::test]
async fn request_timeout_aborts_a_slow_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        .mount(&server)
        .await;

    let mut client = client();
    client.set_retry_policy(RetryPolicy::disabled());
    client.set_request_timeout(Some(Duration::from_millis(50)));
    let request = client.build_request(Method::Get, &format!("{}/resource", server.uri()));

    let error = client.request(request).await.unwrap_err();
    assert!(matches!(
        error,
        everymap_core::error::EveryMapError::TransportError(_)
    ));
}

#[derive(Default)]
struct CapturingObserver {
    request_urls: Mutex<Vec<String>>,
    response_statuses: Mutex<Vec<Option<u16>>>,
}

impl RequestObserver for CapturingObserver {
    fn on_request(&self, observation: &RequestObservation<'_>) {
        self.request_urls
            .lock()
            .unwrap()
            .push(observation.url.to_string());
    }

    fn on_response(&self, observation: &ResponseObservation<'_>) {
        self.response_statuses
            .lock()
            .unwrap()
            .push(observation.status);
    }
}

#[tokio::test]
async fn observer_receives_redacted_urls_and_statuses() {
    let server = MockServer::start().await;
    mount_status(&server, 200).await;

    let observer = Arc::new(CapturingObserver::default());
    let mut client = client();
    client.set_observer(observer.clone());
    let request = client
        .build_request(Method::Get, &format!("{}/resource?q=berlin", server.uri()))
        .query([("key", "another-secret")]);
    client.request(request).await.unwrap();

    let urls = observer.request_urls.lock().unwrap();
    assert_eq!(urls.len(), 1);
    assert!(urls[0].contains("q=berlin"));
    assert!(urls[0].contains("apiKey=***"));
    assert!(urls[0].contains("key=***"));
    assert!(!urls[0].contains("super-secret"));
    assert!(!urls[0].contains("another-secret"));
    drop(urls);

    let statuses = observer.response_statuses.lock().unwrap();
    assert_eq!(statuses.as_slice(), &[Some(200)]);
}

/// A transport that answers immediately, so pacing is the only delay.
#[derive(Default)]
struct InstantTransport {
    calls: Mutex<u32>,
}

#[async_trait::async_trait]
impl everymap_core::http::HttpTransport for InstantTransport {
    async fn send(
        &self,
        _request: HttpRequest,
    ) -> everymap_core::error::EveryMapResult<everymap_core::http::HttpResponse> {
        *self.calls.lock().unwrap() += 1;
        Ok(everymap_core::http::HttpResponse::new(
            200,
            everymap_core::http::HttpHeaders::default(),
            Vec::new(),
        ))
    }
}

#[tokio::test]
async fn token_bucket_paces_successive_requests() {
    let transport = Arc::new(InstantTransport::default());
    let mut client = ProviderClient::with_transport(transport.clone(), api_key_provider(), "test");
    client.set_retry_policy(RetryPolicy::disabled());
    client.set_rate_limiter(Arc::new(TokenBucket::new(1, 50.0)));

    let first = client.build_request(Method::Get, "https://example.com");
    client.request(first).await.unwrap();

    let start = Instant::now();
    let second = client.build_request(Method::Get, "https://example.com");
    client.request(second).await.unwrap();

    assert!(start.elapsed() >= Duration::from_millis(10));
    assert_eq!(*transport.calls.lock().unwrap(), 2);
}

#[test]
fn default_request_carries_the_client_timeout() {
    let mut client = client();
    client.set_request_timeout(Some(Duration::from_secs(9)));
    let request: HttpRequest = client.build_request(Method::Get, "https://example.com");
    assert_eq!(request.timeout_value(), Some(Duration::from_secs(9)));
}
