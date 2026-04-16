use everymap_core::auth::AuthProvider;
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

/// The shared HTTP client for Radar APIs.
///
/// Delegates to `everymap_core::client::ProviderClient` for common
/// request/response logic. Handles authentication via the `Authorization`
/// header (configured via `HeaderAuthProvider`). Each domain module
/// constructs its own URL path per the Radar API specification.
pub struct RadarClient {
    inner: everymap_core::client::ProviderClient,
}

impl RadarClient {
    /// Creates a new `RadarClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            inner: everymap_core::client::ProviderClient::new(auth_provider, "radar"),
        }
    }

    /// Creates a `RadarClient` with a custom `reqwest::Client` configuration.
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> EveryMapResult<Self> {
        Ok(Self {
            inner: everymap_core::client::ProviderClient::with_client_builder(builder, auth_provider, "radar")?,
        })
    }

    /// Enable or disable verbose output.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.inner.set_verbose(verbose);
    }

    /// Whether verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.inner.is_verbose()
    }

    /// Builds a request to the given full URL with the specified HTTP method.
    pub fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.inner.build_request(method, url)
    }

    /// Sends a request, applying authentication first.
    pub async fn request(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response> {
        self.inner.request(builder).await
    }

    /// Sends a request and deserializes the JSON response into `T`.
    pub async fn request_json<T: serde::de::DeserializeOwned>(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<T> {
        self.inner.request_json(builder).await
    }

    /// Sends a POST request with a JSON body and deserializes the response.
    pub async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> EveryMapResult<T> {
        self.inner.post_json(url, body).await
    }
}