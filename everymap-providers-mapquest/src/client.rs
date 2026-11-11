//! The shared MapQuest HTTP client and its configuration.
//!
//! Every MapQuest service is rooted at one platform URL and authenticates with
//! the consumer key as the documented `key` query parameter. [`MapQuestClient`]
//! wraps the workspace [`ProviderClient`] with that credential, the configured
//! base URL and the configured `User-Agent`, so every provider in this crate
//! shares one request pipeline.
//!
//! The key is handed to [`ApiKeyProvider`] and never retained or formatted by
//! this crate. The shared pipeline redacts credential-bearing query values
//! before it reports a request to an observer or in verbose output.

use crate::error_mapping::validate_info_envelope_if_present;
use everymap_core::auth::{ApiKeyProvider, AuthProvider};
use everymap_core::client::ProviderClient;
use everymap_core::error::EveryMapResult;
use everymap_core::http::{HttpRequest, HttpTransport, Method};
use serde_json::Value;
use std::sync::Arc;

/// Name this provider reports in errors and request observations.
pub const MAPQUEST_PROVIDER_NAME: &str = "mapquest";

/// Base URL of the public MapQuest web services platform.
///
/// The directions, geocoding, search, traffic and tiles services are all
/// rooted at this URL; each provider appends its documented service path.
pub const DEFAULT_MAPQUEST_BASE_URL: &str = "https://www.mapquestapi.com";

/// `User-Agent` value sent when the configuration carries none.
pub const DEFAULT_MAPQUEST_USER_AGENT: &str =
    concat!("everymap-mapquest-provider/", env!("CARGO_PKG_VERSION"));

/// Query parameter the platform documents for the consumer key.
pub const MAPQUEST_API_KEY_QUERY_PARAMETER: &str = "key";

/// Header carrying the configured user agent.
const USER_AGENT_HEADER_NAME: &str = "user-agent";

/// Connection settings for every MapQuest provider in this crate.
///
/// The configuration is consumed when a [`MapQuestClient`] is built; the
/// consumer key moves into the client's authentication provider and is not
/// retained by the client afterwards.
///
/// # Fields
/// - `base_url` — platform root without a service path, for example
///   `https://www.mapquestapi.com`. Tests point it at an in-process server.
/// - `api_key` — consumer key issued by the MapQuest developer portal. It is
///   sent as the `key` query parameter and is never written to an error or a
///   log line.
/// - `user_agent` — `User-Agent` header sent with every request. An empty
///   value suppresses the header.
#[derive(Clone, PartialEq, Eq)]
pub struct MapQuestProviderConfiguration {
    /// Platform root URL without a service path.
    pub base_url: String,
    /// Consumer key issued by the MapQuest developer portal.
    pub api_key: String,
    /// `User-Agent` header value sent with every request.
    pub user_agent: String,
}

impl std::fmt::Debug for MapQuestProviderConfiguration {
    /// Prints the connection settings with the consumer key redacted, so a
    /// configuration handed to a logger never prints a credential.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapQuestProviderConfiguration")
            .field("base_url", &self.base_url)
            .field("api_key", &"REDACTED")
            .field("user_agent", &self.user_agent)
            .finish()
    }
}

impl Default for MapQuestProviderConfiguration {
    /// Settings for the public MapQuest platform with no credential.
    ///
    /// A client built with the default configuration cannot authenticate; the
    /// caller supplies the consumer key before sending a request.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_MAPQUEST_BASE_URL.to_string(),
            api_key: String::new(),
            user_agent: DEFAULT_MAPQUEST_USER_AGENT.to_string(),
        }
    }
}

/// Shared HTTP client for the MapQuest web services platform.
///
/// The client holds a [`ProviderClient`] configured with an
/// [`ApiKeyProvider`] and keeps the base URL and user agent every provider
/// needs. Construct it with [`Self::new`] on the target's default transport,
/// or [`Self::with_transport`] to inject one (hosts inject their platform
/// transport and tests inject a fake).
pub struct MapQuestClient {
    inner: ProviderClient,
    base_url: String,
    user_agent: String,
}

impl MapQuestClient {
    /// Builds a client on the target's default transport.
    ///
    /// # Parameters
    /// - `configuration` — platform root, consumer key and user agent.
    ///
    /// # Returns
    /// A client that sends authenticated requests through the platform
    /// transport for the current target.
    pub fn new(configuration: MapQuestProviderConfiguration) -> Self {
        Self::build(configuration, None)
    }

    /// Builds a client that executes requests through an explicit transport.
    ///
    /// # Parameters
    /// - `configuration` — platform root, consumer key and user agent.
    /// - `transport` — the transport that sends each request. The request,
    ///   retry and error-mapping code is identical to the default transport's.
    ///
    /// # Returns
    /// A client that sends authenticated requests through `transport`.
    pub fn with_transport(
        configuration: MapQuestProviderConfiguration,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self::build(configuration, Some(transport))
    }

    /// Shared constructor for both transports.
    fn build(
        configuration: MapQuestProviderConfiguration,
        transport: Option<Arc<dyn HttpTransport>>,
    ) -> Self {
        let auth_provider: Arc<dyn AuthProvider> = Arc::new(ApiKeyProvider::new(
            configuration.api_key,
            MAPQUEST_API_KEY_QUERY_PARAMETER.to_string(),
        ));
        let inner = match transport {
            Some(transport) => {
                ProviderClient::with_transport(transport, auth_provider, MAPQUEST_PROVIDER_NAME)
            }
            None => ProviderClient::new(auth_provider, MAPQUEST_PROVIDER_NAME),
        };
        Self {
            inner,
            base_url: configuration.base_url,
            user_agent: configuration.user_agent,
        }
    }

    /// Installs a token bucket that paces every request this client sends.
    ///
    /// # Parameters
    /// - `rate_limiter` — the bucket shared by the client's requests.
    pub fn set_rate_limiter(&mut self, rate_limiter: Arc<everymap_core::client::TokenBucket>) {
        self.inner.set_rate_limiter(rate_limiter);
    }

    /// Replaces the retry policy applied to idempotent requests.
    ///
    /// # Parameters
    /// - `retry_policy` — the attempt budget and backoff.
    pub fn set_retry_policy(&mut self, retry_policy: everymap_core::client::RetryPolicy) {
        self.inner.set_retry_policy(retry_policy);
    }

    /// Sets the per-request timeout, or removes it.
    ///
    /// # Parameters
    /// - `timeout` — the deadline, or `None` to wait without one.
    pub fn set_request_timeout(&mut self, timeout: Option<std::time::Duration>) {
        self.inner.set_request_timeout(timeout);
    }

    /// Installs an observer that receives every attempt's redacted facts.
    ///
    /// # Parameters
    /// - `observer` — the observer; it replaces any earlier one on this
    ///   client and takes precedence over a process-wide observer.
    pub fn set_observer(&mut self, observer: Arc<dyn everymap_core::client::RequestObserver>) {
        self.inner.set_observer(observer);
    }

    /// The platform root URL requests are built against.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The `User-Agent` value every request carries.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Enables or disables verbose request and response logging.
    ///
    /// Verbose output is written to standard error and carries the redacted
    /// request URL and the response body; the consumer key is never part of
    /// it because the shared pipeline redacts credential query parameters.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.inner.set_verbose(verbose);
    }

    /// Whether verbose diagnostics are enabled.
    pub fn is_verbose(&self) -> bool {
        self.inner.is_verbose()
    }

    /// Sends one request through the shared pipeline.
    ///
    /// The request must be built through [`Self::build_request`] so that the
    /// base URL and user agent are already applied. Authentication is added by
    /// the client's auth provider.
    ///
    /// # Errors
    /// Returns the shared client's HTTP, rate-limit or transport error when no
    /// successful response arrives.
    pub(crate) async fn request(
        &self,
        request: HttpRequest,
    ) -> EveryMapResult<everymap_core::http::HttpResponse> {
        self.inner.request(request).await
    }

    /// Builds a request for a service path.
    ///
    /// # Parameters
    /// - `method` — HTTP method of the service endpoint.
    /// - `path` — service path without a leading slash, for example
    ///   `geocoding/v1/address`.
    /// - `query` — query parameters the endpoint documents, without the
    ///   credential (the client's auth provider appends it).
    ///
    /// # Returns
    /// A request against `{base_url}/{path}` carrying the configured user
    /// agent and the client's request timeout.
    pub(crate) fn build_request(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
    ) -> HttpRequest {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let request = self.inner.build_request(method, &url).query(query);
        if self.user_agent.is_empty() {
            request
        } else {
            request.header(USER_AGENT_HEADER_NAME, &self.user_agent)
        }
    }

    /// Fetches a JSON payload from a service path.
    ///
    /// A successfully decoded body is checked against the `info` envelope when
    /// one is present; an application-level failure becomes a provider error.
    ///
    /// # Parameters
    /// - `path` — service path without a leading slash.
    /// - `query` — documented query parameters, without the credential.
    ///
    /// # Returns
    /// The decoded JSON payload after a successful envelope check.
    ///
    /// # Errors
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when the body is not JSON.
    /// - [`EveryMapError::ProviderError`] when the `info` envelope reports a
    ///   non-zero status code.
    pub(crate) async fn get_json(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> EveryMapResult<Value> {
        let request = self.build_request(Method::Get, path, query);
        let response = self.request(request).await?;
        let payload: Value = serde_json::from_str(&response.text())?;
        validate_info_envelope_if_present(&payload)?;
        Ok(payload)
    }

    /// Posts a JSON body to a service path and returns the decoded payload.
    ///
    /// # Parameters
    /// - `path` — service path without a leading slash.
    /// - `query` — documented query parameters, without the credential.
    /// - `body` — request body serialized as JSON.
    ///
    /// # Returns
    /// The decoded JSON payload after a successful envelope check.
    ///
    /// # Errors
    /// - [`EveryMapError::SerializationError`] when the body cannot be
    ///   serialized or the response body is not JSON.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::ProviderError`] when the `info` envelope reports a
    ///   non-zero status code.
    pub(crate) async fn post_json(
        &self,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> EveryMapResult<Value> {
        let request = self.build_request(Method::Post, path, query).json(body)?;
        let response = self.request(request).await?;
        let payload: Value = serde_json::from_str(&response.text())?;
        validate_info_envelope_if_present(&payload)?;
        Ok(payload)
    }

    /// Fetches raw bytes from a service path.
    ///
    /// Raster tiles carry no JSON envelope, so a successful response returns
    /// its body verbatim.
    ///
    /// # Parameters
    /// - `path` — service path without a leading slash.
    ///
    /// # Returns
    /// The response body bytes.
    ///
    /// # Errors
    /// Returns the shared client's HTTP, rate-limit or transport error for an
    /// unsuccessful response.
    pub(crate) async fn get_bytes(&self, path: &str) -> EveryMapResult<Vec<u8>> {
        let request = self.build_request(Method::Get, path, &[]);
        Ok(self.request(request).await?.into_body())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MapQuestClient, MapQuestProviderConfiguration, DEFAULT_MAPQUEST_BASE_URL,
        MAPQUEST_API_KEY_QUERY_PARAMETER,
    };
    use async_trait::async_trait;
    use everymap_core::error::EveryMapResult;
    use everymap_core::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
    use std::sync::{Arc, Mutex};

    const TEST_API_KEY: &str = "test-placeholder-api-key";

    /// Records every request and answers one fixed JSON payload.
    struct RecordingTransport {
        requests: Mutex<Vec<HttpRequest>>,
        body: Vec<u8>,
    }

    #[async_trait]
    impl HttpTransport for RecordingTransport {
        async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
            self.requests
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(request);
            Ok(HttpResponse::new(
                200,
                HttpHeaders::default(),
                self.body.clone(),
            ))
        }
    }

    fn configuration() -> MapQuestProviderConfiguration {
        MapQuestProviderConfiguration {
            base_url: "https://mapquest.example.test".to_string(),
            api_key: TEST_API_KEY.to_string(),
            user_agent: "everymap-mapquest-tests".to_string(),
        }
    }

    #[test]
    fn default_configuration_targets_the_public_platform() {
        let configuration = MapQuestProviderConfiguration::default();
        assert_eq!(configuration.base_url, DEFAULT_MAPQUEST_BASE_URL);
        assert!(configuration.api_key.is_empty());
        assert!(!configuration.user_agent.is_empty());
    }

    #[test]
    fn the_credential_parameter_name_is_the_documented_key() {
        assert_eq!(MAPQUEST_API_KEY_QUERY_PARAMETER, "key");
    }

    /// A configuration handed to a logger must not print the consumer key.
    #[test]
    fn the_configuration_debug_redacts_the_consumer_key() {
        let configuration = configuration();
        let debug = format!("{configuration:?}");
        assert!(!debug.contains(TEST_API_KEY), "{debug}");
        assert!(debug.contains("https://mapquest.example.test"), "{debug}");
        assert!(debug.contains("REDACTED"), "{debug}");
    }

    #[tokio::test]
    async fn a_request_carries_the_base_url_the_user_agent_and_the_key_parameter() {
        let transport = Arc::new(RecordingTransport {
            requests: Mutex::new(Vec::new()),
            body: br#"{"info":{"statuscode":0,"messages":[]},"results":[]}"#.to_vec(),
        });
        let client = MapQuestClient::with_transport(configuration(), transport.clone());
        client
            .get_json(
                "geocoding/v1/address",
                &[("location".to_string(), "Denver".to_string())],
            )
            .await
            .expect("the fixed transport answers a valid payload");

        let requests = transport
            .requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(
            request.url(),
            "https://mapquest.example.test/geocoding/v1/address"
        );
        assert_eq!(
            request.headers(),
            &[(
                "user-agent".to_string(),
                "everymap-mapquest-tests".to_string()
            )]
        );
        let url_with_query = request.url_with_query();
        assert!(url_with_query.contains("location=Denver"));
        assert!(url_with_query.contains("key=test-placeholder-api-key"));
    }

    #[test]
    fn the_base_url_and_user_agent_are_readable() {
        let client = MapQuestClient::new(configuration());
        assert_eq!(client.base_url(), "https://mapquest.example.test");
        assert_eq!(client.user_agent(), "everymap-mapquest-tests");
        assert!(!client.is_verbose());
    }

    #[test]
    fn the_request_pipeline_can_be_reconfigured() {
        let mut client = MapQuestClient::new(configuration());
        client.set_rate_limiter(Arc::new(everymap_core::client::TokenBucket::new(30, 5.0)));
        client.set_retry_policy(everymap_core::client::RetryPolicy::disabled());
        client.set_request_timeout(Some(std::time::Duration::from_secs(10)));
        client.set_request_timeout(None);
    }
}
