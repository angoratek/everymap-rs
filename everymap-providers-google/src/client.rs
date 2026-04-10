use everymap_core::auth::AuthProvider;
use everymap_core::error::{EveryMapError, EveryMapResult};
use reqwest::Client;
use std::sync::Arc;

/// The shared HTTP client for Google Maps APIs.
///
/// Handles authentication via API key injection. Each domain module
/// constructs its own base URL per the Google Maps API specification.
///
/// Supported APIs:
/// - Geocoding API (geocoding, reverse geocoding)
/// - Directions API (routing)
/// - Roads API (route matching) — planned
/// - Static Maps API (map images) — planned
pub struct GoogleClient {
    http_client: Client,
    auth_provider: Arc<dyn AuthProvider>,
}

impl GoogleClient {
    /// Creates a new `GoogleClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            http_client: Client::new(),
            auth_provider,
        }
    }

    /// Creates a `GoogleClient` with a custom `reqwest::Client` configuration.
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> EveryMapResult<Self> {
        Ok(Self {
            http_client: builder.build()?,
            auth_provider,
        })
    }

    /// Builds a request to the given full URL with the specified HTTP method.
    pub fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.http_client.request(method, url)
    }

    /// Sends a request, applying authentication first.
    ///
    /// Returns an `EveryMapError::HttpError` for non-2xx status codes,
    /// and `EveryMapError::RateLimited` for 429 responses.
    pub async fn request(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response> {
        let builder = self.auth_provider.apply(builder).await?;
        let response: reqwest::Response = builder.send().await?;
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            let status_code = status.as_u16();
            let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();
            let retry_after = if status_code == 429 {
                response.headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
            } else {
                None
            };
            let body = response.text().await.unwrap_or_default();
            if status_code == 429 {
                Err(EveryMapError::rate_limited("google", retry_after))
            } else {
                Err(EveryMapError::http_with_body(
                    status_code,
                    format!("HTTP error: {}", status_text),
                    body,
                ))
            }
        }
    }
}