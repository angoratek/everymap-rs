use everymap_core::auth::AuthProvider;
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

/// The shared HTTP client for TomTom APIs.
///
/// Delegates to `everymap_core::client::ProviderClient` for common
/// request/response logic. Each domain module constructs its own
/// base URL per the TomTom API specification.
pub struct TomTomClient {
    inner: everymap_core::client::ProviderClient,
}

impl TomTomClient {
    /// Creates a new `TomTomClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            inner: everymap_core::client::ProviderClient::new(auth_provider, "tomtom"),
        }
    }

    /// Creates a `TomTomClient` with a custom `reqwest::Client` configuration.
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> EveryMapResult<Self> {
        Ok(Self {
            inner: everymap_core::client::ProviderClient::with_client_builder(
                builder,
                auth_provider,
                "tomtom",
            )?,
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
    pub async fn request(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> EveryMapResult<reqwest::Response> {
        self.inner.request(builder).await
    }

    /// Sends a request and deserializes the JSON response into `T`.
    pub async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> EveryMapResult<T> {
        self.inner.request_json(builder).await
    }
}
