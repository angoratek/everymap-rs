//! Tests for the Photon client's shared request-pipeline setters.
//!
//! Pacing, retries, timeouts and observation belong to the shared pipeline, so
//! each setter must reach every Photon request. The tests drive a real Photon
//! provider over an in-process transport; no test reaches an external service.

use async_trait::async_trait;
use everymap_core::client::{
    RequestObservation, RequestObserver, ResponseObservation, RetryPolicy, TokenBucket,
};
use everymap_core::domains::search::{GeocodeOptions, Geocoder};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
use everymap_providers_photon::{
    PhotonClient, PhotonGeocoder, PhotonProviderConfiguration, DEFAULT_PHOTON_USER_AGENT,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A valid GeoJSON answer with no features, enough to complete a query.
const EMPTY_FEATURE_COLLECTION: &str = r#"{"type":"FeatureCollection","features":[]}"#;

/// Test provider configuration naming the provider's own user agent.
fn configuration() -> PhotonProviderConfiguration {
    PhotonProviderConfiguration {
        base_url: "https://photon.example.test".to_string(),
        user_agent: DEFAULT_PHOTON_USER_AGENT.to_string(),
    }
}

/// One request seen by the recording transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordedRequest {
    timeout: Option<Duration>,
}

/// Transport that records each request's timeout and answers with an empty
/// feature collection.
#[derive(Default)]
struct RecordingTransport {
    requests: Mutex<Vec<RecordedRequest>>,
}

impl RecordingTransport {
    fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

#[async_trait]
impl HttpTransport for RecordingTransport {
    async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(RecordedRequest {
                timeout: request.timeout_value(),
            });
        Ok(HttpResponse::new(
            200,
            HttpHeaders::default(),
            EMPTY_FEATURE_COLLECTION.as_bytes().to_vec(),
        ))
    }
}

/// Transport that answers every request with HTTP 500 and counts the attempts.
#[derive(Default)]
struct FailingTransport {
    attempts: AtomicUsize,
}

impl FailingTransport {
    fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl HttpTransport for FailingTransport {
    async fn send(&self, _request: HttpRequest) -> EveryMapResult<HttpResponse> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(HttpResponse::new(
            500,
            HttpHeaders::default(),
            b"server error".to_vec(),
        ))
    }
}

/// Observer that records the provider and attempt number of each request.
#[derive(Default)]
struct RecordingObserver {
    provider_attempts: Mutex<Vec<String>>,
}

impl RecordingObserver {
    fn provider_attempts(&self) -> Vec<String> {
        self.provider_attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl RequestObserver for RecordingObserver {
    fn on_request(&self, observation: &RequestObservation<'_>) {
        self.provider_attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("{} {}", observation.provider, observation.attempt));
    }

    fn on_response(&self, _observation: &ResponseObservation<'_>) {}
}

#[tokio::test]
async fn the_request_timeout_setter_reaches_every_request() {
    let transport = Arc::new(RecordingTransport::default());
    let mut client = PhotonClient::with_transport(configuration(), transport.clone());
    client.set_request_timeout(Some(Duration::from_secs(7)));
    let geocoder = PhotonGeocoder::new(Arc::new(client));

    geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect("the empty feature collection decodes");

    assert_eq!(
        transport.requests()[0].timeout,
        Some(Duration::from_secs(7))
    );
}

#[tokio::test]
async fn the_retry_policy_setter_controls_the_attempt_budget() {
    let transport = Arc::new(FailingTransport::default());
    let mut client = PhotonClient::with_transport(configuration(), transport.clone());
    client.set_retry_policy(RetryPolicy::disabled());
    let geocoder = PhotonGeocoder::new(Arc::new(client));

    let error = geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect_err("a server error surfaces");
    assert!(matches!(
        error,
        EveryMapError::HttpError { status: 500, .. }
    ));
    assert_eq!(
        transport.attempts(),
        1,
        "the disabled policy sends exactly one attempt"
    );
}

#[tokio::test]
async fn the_rate_limiter_setter_refuses_a_request_past_the_declared_wait() {
    let transport = Arc::new(RecordingTransport::default());
    let mut client = PhotonClient::with_transport(configuration(), transport.clone());
    client.set_rate_limiter(Arc::new(
        TokenBucket::new(1, 0.001).with_maximum_wait(Duration::from_millis(1)),
    ));
    let geocoder = PhotonGeocoder::new(Arc::new(client));

    geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect("the one-token burst admits the first request");
    let error = geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect_err("the bucket refuses a wait past its ceiling");
    assert!(matches!(error, EveryMapError::RateLimited { .. }));
    assert_eq!(
        transport.requests().len(),
        1,
        "the refused request never reaches the transport"
    );
}

#[tokio::test]
async fn the_observer_setter_receives_every_attempt() {
    let transport = Arc::new(RecordingTransport::default());
    let observer = Arc::new(RecordingObserver::default());
    let mut client = PhotonClient::with_transport(configuration(), transport.clone());
    client.set_observer(observer.clone());
    let geocoder = PhotonGeocoder::new(Arc::new(client));

    geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect("the empty feature collection decodes");

    assert_eq!(observer.provider_attempts(), vec!["photon 1".to_string()]);
}
