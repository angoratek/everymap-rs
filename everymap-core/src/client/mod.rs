//! Shared provider HTTP client.
//!
//! [`ProviderClient`] owns the request pipeline used by every provider crate:
//! authentication, optional rate limiting, transport dispatch, retry for
//! idempotent requests, timeout enforcement, and response/error mapping. The
//! actual HTTP backend is any [`crate::http::HttpTransport`].

mod macros;
mod observer;
mod rate_limit;
mod retry;
mod sleep;

use crate::auth::AuthProvider;
use crate::error::{EveryMapError, EveryMapResult};
use crate::http::{default_transport, HttpRequest, HttpResponse, Method};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use web_time::Instant;

pub use observer::{
    is_sensitive_header, redact_url, RequestObservation, RequestObserver, ResponseObservation,
    REDACTED_HEADER_NAMES, REDACTED_QUERY_PARAMETERS, REDACTED_VALUE,
};
pub use rate_limit::TokenBucket;
/// The native client builder a provider exposes through
/// `with_client_builder`, re-exported so a provider crate does not have to
/// depend on `reqwest` itself. Absent in the browser, where the fetch
/// transport has no builder to configure.
#[cfg(not(target_arch = "wasm32"))]
pub use reqwest::ClientBuilder;
pub use retry::RetryPolicy;

/// The process-wide request observer, installed once by a host binding.
static GLOBAL_REQUEST_OBSERVER: OnceLock<Mutex<Option<Arc<dyn RequestObserver>>>> = OnceLock::new();

/// The slot holding the process-wide observer.
fn global_request_observer_slot() -> &'static Mutex<Option<Arc<dyn RequestObserver>>> {
    GLOBAL_REQUEST_OBSERVER.get_or_init(|| Mutex::new(None))
}

/// Installs a process-wide observer for every provider client.
///
/// A host binding that wants request diagnostics installs one observer here
/// instead of threading it through every provider constructor. A client's own
/// observer, when set with [`ProviderClient::set_observer`], takes precedence.
///
/// # Parameters
/// - `observer` — the observer to call for each attempt.
pub fn install_global_request_observer(observer: Arc<dyn RequestObserver>) {
    *global_request_observer_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(observer);
}

/// Removes the process-wide observer, leaving each client's own observer in
/// place.
pub fn clear_global_request_observer() {
    *global_request_observer_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// The installed process-wide observer, when one is set.
fn global_request_observer() -> Option<Arc<dyn RequestObserver>> {
    global_request_observer_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Maximum bytes of response body to display in verbose mode.
const VERBOSE_BODY_LIMIT: usize = 10_240;

/// Maximum bytes of response body included in deserialization error messages.
const ERROR_BODY_LIMIT: usize = 256;

/// HTTP header name for rate-limit retry indication.
const RETRY_AFTER_HEADER: &str = "retry-after";

/// Error code for JSON deserialization failures.
const DESERIALIZATION_ERROR: &str = "DESERIALIZATION_ERROR";

/// Default per-request timeout when the caller does not set one.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Longest `Retry-After` the client waits on a `429` before returning it.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(5);

/// HTTP status code for rate limiting.
const STATUS_TOO_MANY_REQUESTS: u16 = 429;

/// Lower bound of the retryable server-error range.
const STATUS_SERVER_ERROR_START: u16 = 500;

/// Upper bound of the retryable server-error range.
const STATUS_SERVER_ERROR_END: u16 = 600;

/// Status text used when the status code has no known phrase.
const UNKNOWN_STATUS: &str = "Unknown";

/// A provider HTTP client that consolidates the common request/response logic
/// shared across all provider implementations (HERE, Google, TomTom, MapBox,
/// Radar).
///
/// The client is backend-agnostic: requests are [`HttpRequest`] values sent
/// through an [`crate::http::HttpTransport`]. Native builds default to
/// `reqwest`; WebAssembly builds default to `window.fetch`.
pub struct ProviderClient {
    transport: Arc<dyn crate::http::HttpTransport>,
    auth_provider: Arc<dyn AuthProvider>,
    verbose: bool,
    provider_name: &'static str,
    request_timeout: Option<Duration>,
    retry_policy: RetryPolicy,
    rate_limiter: Option<Arc<TokenBucket>>,
    observer: Option<Arc<dyn RequestObserver>>,
}

impl ProviderClient {
    /// Creates a new `ProviderClient` with the given authentication provider
    /// and the target's default transport.
    pub fn new(auth_provider: Arc<dyn AuthProvider>, provider_name: &'static str) -> Self {
        Self::with_transport(default_transport(), auth_provider, provider_name)
    }

    /// Creates a `ProviderClient` with an explicit transport.
    pub fn with_transport(
        transport: Arc<dyn crate::http::HttpTransport>,
        auth_provider: Arc<dyn AuthProvider>,
        provider_name: &'static str,
    ) -> Self {
        Self {
            transport,
            auth_provider,
            verbose: false,
            provider_name,
            request_timeout: Some(DEFAULT_REQUEST_TIMEOUT),
            retry_policy: RetryPolicy::default(),
            rate_limiter: None,
            observer: None,
        }
    }

    /// Creates a `ProviderClient` with a custom `reqwest::Client` configuration.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
        provider_name: &'static str,
    ) -> EveryMapResult<Self> {
        let client = builder.build()?;
        Ok(Self::with_transport(
            Arc::new(crate::http::ReqwestTransport::with_client(client)),
            auth_provider,
            provider_name,
        ))
    }

    /// Enable or disable verbose output (request/response logging to stderr).
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Whether verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// The provider name (e.g., "here", "google", "tomtom", "mapbox", "radar").
    pub fn provider_name(&self) -> &'static str {
        self.provider_name
    }

    /// Override the timeout applied to each request attempt.
    pub fn set_request_timeout(&mut self, timeout: Option<Duration>) {
        self.request_timeout = timeout;
    }

    /// Replace the retry policy.
    pub fn set_retry_policy(&mut self, retry_policy: RetryPolicy) {
        self.retry_policy = retry_policy;
    }

    /// Pace requests through a token bucket.
    pub fn set_rate_limiter(&mut self, rate_limiter: Arc<TokenBucket>) {
        self.rate_limiter = Some(rate_limiter);
    }

    /// Install an observer that receives redacted request and response data.
    pub fn set_observer(&mut self, observer: Arc<dyn RequestObserver>) {
        self.observer = Some(observer);
    }

    /// Builds a request to the given full URL with the specified HTTP method.
    ///
    /// The client's request timeout is applied unless the caller overrides it
    /// with [`HttpRequest::timeout`].
    pub fn build_request(&self, method: Method, url: &str) -> HttpRequest {
        let request = HttpRequest::new(method, url);
        match self.request_timeout {
            Some(timeout) => request.timeout(timeout),
            None => request,
        }
    }

    /// Sends a request, applying authentication, pacing, retries, and
    /// response mapping.
    ///
    /// Returns `EveryMapError::HttpError` for non-2xx status codes,
    /// `EveryMapError::RateLimited` for 429 responses, and
    /// `EveryMapError::ClientError`/`EveryMapError::TransportError` for
    /// transport failures.
    pub async fn request(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
        let request = self.auth_provider.apply(request).await?;
        let redacted_url = redact_url(&request.url_with_query());
        let max_attempts = if request.method().is_idempotent() {
            self.retry_policy.max_attempts.max(1)
        } else {
            1
        };

        let mut last_error = EveryMapError::Unknown;
        let observer = self.observer.clone().or_else(global_request_observer);
        for attempt in 1..=max_attempts {
            if let Some(rate_limiter) = &self.rate_limiter {
                if !rate_limiter.acquire().await {
                    return Err(EveryMapError::RateLimited {
                        provider: self.provider_name.to_string(),
                        retry_after_secs: None,
                    });
                }
            }
            if let Some(observer) = &observer {
                observer.on_request(&RequestObservation {
                    provider: self.provider_name,
                    method: request.method(),
                    url: &redacted_url,
                    attempt,
                });
            }

            let start = Instant::now();
            match self.transport.send(request.clone()).await {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    if let Some(observer) = &observer {
                        observer.on_response(&ResponseObservation {
                            provider: self.provider_name,
                            status: Some(response.status()),
                            elapsed,
                            attempt,
                        });
                    }
                    if response.is_success() {
                        if self.verbose {
                            eprintln!(
                                "[VERBOSE] {} {} — {} ({:.0}ms)",
                                response.status(),
                                redacted_url,
                                status_text(response.status()),
                                elapsed.as_secs_f64() * 1000.0
                            );
                        }
                        return Ok(response);
                    }

                    let status = response.status();
                    let retry_after = if status == STATUS_TOO_MANY_REQUESTS {
                        response
                            .headers()
                            .get(RETRY_AFTER_HEADER)
                            .and_then(|value| value.trim().parse::<u64>().ok())
                    } else {
                        None
                    };
                    let body = response.text();
                    if self.verbose {
                        eprintln!(
                            "[VERBOSE] {} {} — {} ({:.0}ms)",
                            status,
                            redacted_url,
                            status_text(status),
                            elapsed.as_secs_f64() * 1000.0
                        );
                        eprintln!(
                            "[VERBOSE] Error response body:\n{}",
                            truncate_str(&body, VERBOSE_BODY_LIMIT)
                        );
                    }

                    let error = if status == STATUS_TOO_MANY_REQUESTS {
                        EveryMapError::rate_limited(self.provider_name, retry_after)
                    } else {
                        EveryMapError::http_with_body(
                            status,
                            format!("HTTP error: {}", status_text(status)),
                            body,
                        )
                    };

                    if attempt < max_attempts && self.should_retry(status, retry_after) {
                        let delay = retry_after
                            .map(Duration::from_secs)
                            .unwrap_or_else(|| self.retry_policy.delay_after_attempt(attempt));
                        sleep::sleep(delay).await;
                        last_error = error;
                        continue;
                    }
                    return Err(error);
                }
                Err(error) => {
                    let elapsed = start.elapsed();
                    if let Some(observer) = &self.observer {
                        observer.on_response(&ResponseObservation {
                            provider: self.provider_name,
                            status: None,
                            elapsed,
                            attempt,
                        });
                    }
                    if attempt < max_attempts {
                        sleep::sleep(self.retry_policy.delay_after_attempt(attempt)).await;
                        last_error = error;
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        Err(last_error)
    }

    /// Whether a response status is retryable for an idempotent request.
    fn should_retry(&self, status: u16, retry_after: Option<u64>) -> bool {
        if status == STATUS_TOO_MANY_REQUESTS {
            return retry_after
                .is_none_or(|seconds| Duration::from_secs(seconds) <= MAX_RETRY_AFTER);
        }
        (STATUS_SERVER_ERROR_START..STATUS_SERVER_ERROR_END).contains(&status)
    }

    /// Sends a request and deserializes the JSON response into `T`.
    ///
    /// Reads the response body as text first, then deserializes with
    /// `serde_json::from_str`. If deserialization fails, the error includes
    /// a truncated excerpt of the raw response body for debugging.
    pub async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        request: HttpRequest,
    ) -> EveryMapResult<T> {
        let start = Instant::now();
        let response = self.request(request).await?;
        let body = response.text();

        if self.verbose {
            eprintln!(
                "[VERBOSE] Response body ({:.0}ms, {} bytes):\n{}",
                start.elapsed().as_secs_f64() * 1000.0,
                body.len(),
                truncate_str(&body, VERBOSE_BODY_LIMIT)
            );
        }

        serde_json::from_str::<T>(&body).map_err(|e| {
            let snippet = truncate_str(&body, ERROR_BODY_LIMIT);
            EveryMapError::provider(
                self.provider_name,
                DESERIALIZATION_ERROR,
                format!(
                    "Failed to deserialize response: {}\nResponse body (first {} bytes): {}",
                    e, ERROR_BODY_LIMIT, snippet
                ),
            )
        })
    }

    /// Sends a POST request with a JSON body and deserializes the response.
    pub async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> EveryMapResult<T> {
        let request = self.build_request(Method::Post, url).json(body)?;
        self.request_json(request).await
    }
}

/// Human-readable phrase for common HTTP status codes.
fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => UNKNOWN_STATUS,
    }
}

/// Backward-compatible alias for [`redact_url`].
pub fn redact_api_key(url: &str) -> String {
    redact_url(url)
}

/// Truncate a string to the given byte limit, appending info if truncated.
pub fn truncate_str(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .take_while(|(byte_index, _)| *byte_index < limit)
            .last()
            .map(|(byte_index, char_value)| byte_index + char_value.len_utf8())
            .unwrap_or(limit.min(s.len()));
        format!("{}...\n[truncated, {} bytes total]", &s[..end], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::NoAuthProvider;
    use crate::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A transport that always answers an empty success.
    struct FixedSuccessTransport;

    #[async_trait]
    impl HttpTransport for FixedSuccessTransport {
        async fn send(&self, _request: HttpRequest) -> EveryMapResult<HttpResponse> {
            Ok(HttpResponse::new(
                200,
                HttpHeaders::new(Vec::new()),
                Vec::new(),
            ))
        }
    }

    /// Counts the observation callbacks it receives.
    #[derive(Default)]
    struct CountingRequestObserver {
        requests: AtomicUsize,
        responses: AtomicUsize,
    }

    impl RequestObserver for CountingRequestObserver {
        fn on_request(&self, _observation: &RequestObservation<'_>) {
            self.requests.fetch_add(1, Ordering::Relaxed);
        }

        fn on_response(&self, _observation: &ResponseObservation<'_>) {
            self.responses.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[tokio::test]
    async fn a_global_observer_sees_every_client_request() {
        let observer = Arc::new(CountingRequestObserver::default());
        install_global_request_observer(observer.clone());
        let client = ProviderClient::with_transport(
            Arc::new(FixedSuccessTransport),
            Arc::new(NoAuthProvider::new()),
            "global-observer-test",
        );
        client
            .request(HttpRequest::get("https://example.com/probe"))
            .await
            .expect("the fixed transport answers");
        clear_global_request_observer();
        assert_eq!(observer.requests.load(Ordering::Relaxed), 1);
        assert_eq!(observer.responses.load(Ordering::Relaxed), 1);
        assert!(global_request_observer().is_none());
    }

    #[test]
    fn test_redact_api_key_here() {
        let url = "https://router.hereapi.com/v8/routes?apiKey=secret123&origin=52.5,13.3";
        assert_eq!(
            redact_api_key(url),
            "https://router.hereapi.com/v8/routes?apiKey=***&origin=52.5,13.3"
        );
    }

    #[test]
    fn test_redact_api_key_google() {
        let url = "https://maps.googleapis.com/maps/api/geocode/json?key=secret123&address=Berlin";
        assert_eq!(
            redact_api_key(url),
            "https://maps.googleapis.com/maps/api/geocode/json?key=***&address=Berlin"
        );
    }

    #[test]
    fn test_redact_api_key_mapbox() {
        let url = "https://api.mapbox.com/geocode/v6/forward?q=test&access_token=pk.secret123";
        assert_eq!(
            redact_api_key(url),
            "https://api.mapbox.com/geocode/v6/forward?q=test&access_token=***"
        );
    }

    #[test]
    fn test_redact_no_key() {
        let url = "https://example.com/api?foo=bar";
        assert_eq!(redact_api_key(url), url);
    }

    #[test]
    fn test_truncate_str_short() {
        let s = "hello";
        assert_eq!(truncate_str(s, 100), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        let s = "a".repeat(20000);
        let truncated = truncate_str(&s, 1024);
        assert!(truncated.len() < s.len());
        assert!(truncated.contains("[truncated"));
    }

    #[test]
    fn test_provider_client_new() {
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new(
            "test".to_string(),
            "key".to_string(),
        ));
        let client = ProviderClient::new(auth, "test");
        assert_eq!(client.provider_name(), "test");
        assert!(!client.is_verbose());
        assert_eq!(client.request_timeout, Some(DEFAULT_REQUEST_TIMEOUT));
        assert!(client.retry_policy.is_enabled());
    }

    #[test]
    fn test_provider_client_verbose() {
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new(
            "test".to_string(),
            "key".to_string(),
        ));
        let mut client = ProviderClient::new(auth, "test");
        client.set_verbose(true);
        assert!(client.is_verbose());
    }

    #[test]
    fn build_request_applies_client_timeout() {
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new(
            "test".to_string(),
            "key".to_string(),
        ));
        let mut client = ProviderClient::new(auth, "test");
        client.set_request_timeout(Some(Duration::from_secs(7)));
        let request = client.build_request(Method::Get, "https://example.com");
        assert_eq!(request.timeout_value(), Some(Duration::from_secs(7)));
    }

    #[test]
    fn retry_status_classification() {
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new(
            "test".to_string(),
            "key".to_string(),
        ));
        let client = ProviderClient::new(auth, "test");
        assert!(client.should_retry(503, None));
        assert!(client.should_retry(429, None));
        assert!(client.should_retry(429, Some(2)));
        assert!(!client.should_retry(429, Some(60)));
        assert!(!client.should_retry(400, None));
        assert!(!client.should_retry(401, None));
    }

    #[test]
    fn status_text_covers_common_codes() {
        assert_eq!(status_text(401), "Unauthorized");
        assert_eq!(status_text(429), "Too Many Requests");
        assert_eq!(status_text(599), UNKNOWN_STATUS);
    }
}
