use everymap_core::auth::AuthProvider;
use everymap_core::error::EveryMapResult;
use reqwest::Client;
use std::sync::Arc;

/// The shared HTTP client for HERE Technologies APIs.
///
/// This client is a thin wrapper around `reqwest::Client` that handles
/// authentication. Each domain module constructs its own base URL per the
/// HERE API specification.
pub struct HereClient {
    http_client: Client,
    auth_provider: Arc<dyn AuthProvider>,
}

impl HereClient {
    /// Creates a new `HereClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            http_client: Client::new(),
            auth_provider,
        }
    }

    /// Builds a request to the given full URL with the specified HTTP method.
    pub fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.http_client.request(method, url)
    }

    /// Sends a request, applying authentication first.
    pub async fn request(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response> {
        let builder = self.auth_provider.apply(builder).await?;
        builder.send().await.map_err(Into::into)
    }
}