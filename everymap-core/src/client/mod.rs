use crate::auth::AuthProvider;
use crate::error::{EveryMapError, EveryMapResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// Maximum bytes of response body to display in verbose mode.
const VERBOSE_BODY_LIMIT: usize = 10_240;

/// Maximum bytes of response body included in deserialization error messages.
const ERROR_BODY_LIMIT: usize = 256;

/// Trait for HTTP clients, allowing dependency injection and testing.
///
/// The default implementation wraps `reqwest::Client`. Custom implementations
/// can be used for testing (mock clients), retry logic, or custom transport
/// configuration (timeouts, proxies, connection pools).
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request and return the response.
    async fn send(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response>;
}

/// Default HTTP client that wraps `reqwest::Client`.
pub struct DefaultHttpClient {
    #[allow(dead_code)] // Used by HttpClient impl
    inner: reqwest::Client,
}

impl DefaultHttpClient {
    /// Create a new default HTTP client with default configuration.
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    /// Create a default HTTP client with custom configuration.
    pub fn with_config(builder: reqwest::ClientBuilder) -> EveryMapResult<Self> {
        Ok(Self {
            inner: builder.build()?,
        })
    }
}

impl Default for DefaultHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for DefaultHttpClient {
    async fn send(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response> {
        builder.send().await.map_err(Into::into)
    }
}

/// A generic provider HTTP client that consolidates the common request/response logic
/// shared across all provider implementations (HERE, Google, TomTom, MapBox, Radar).
///
/// Each provider crate should use this instead of maintaining its own duplicated client.
/// The only configuration specific to each provider is the `provider_name` (used in
/// error messages) and the auth mechanism (injected via `AuthProvider`).
pub struct ProviderClient {
    http_client: reqwest::Client,
    auth_provider: Arc<dyn AuthProvider>,
    verbose: bool,
    provider_name: &'static str,
}

impl ProviderClient {
    /// Creates a new `ProviderClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>, provider_name: &'static str) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            auth_provider,
            verbose: false,
            provider_name,
        }
    }

    /// Creates a `ProviderClient` with a custom `reqwest::Client` configuration.
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
        provider_name: &'static str,
    ) -> EveryMapResult<Self> {
        Ok(Self {
            http_client: builder.build()?,
            auth_provider,
            verbose: false,
            provider_name,
        })
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

    /// Builds a request to the given full URL with the specified HTTP method.
    pub fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.http_client.request(method, url)
    }

    /// Sends a request, applying authentication first.
    ///
    /// Returns an `EveryMapError::HttpError` for non-2xx status codes,
    /// and `EveryMapError::RateLimited` for 429 responses.
    pub async fn request(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<reqwest::Response> {
        let start = Instant::now();
        let builder = self.auth_provider.apply(builder).await?;
        let response: reqwest::Response = builder.send().await?;
        let elapsed = start.elapsed();
        let status = response.status();

        if self.verbose {
            let url = response.url().to_string();
            eprintln!(
                "[VERBOSE] {} {} — {} ({:.0}ms)",
                status.as_u16(),
                redact_api_key(&url),
                status.canonical_reason().unwrap_or("Unknown"),
                elapsed.as_secs_f64() * 1000.0
            );
        }

        if status.is_success() {
            Ok(response)
        } else {
            let status_code = status.as_u16();
            let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();
            let retry_after = if status_code == 429 {
                response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
            } else {
                None
            };
            let body = response.text().await.unwrap_or_default();

            if self.verbose {
                let snippet = truncate_str(&body, VERBOSE_BODY_LIMIT);
                eprintln!("[VERBOSE] Error response body:\n{}", snippet);
            }

            if status_code == 429 {
                Err(EveryMapError::rate_limited(self.provider_name, retry_after))
            } else {
                Err(EveryMapError::http_with_body(
                    status_code,
                    format!("HTTP error: {}", status_text),
                    body,
                ))
            }
        }
    }

    /// Sends a request and deserializes the JSON response into `T`.
    ///
    /// Reads the response body as text first, then deserializes with
    /// `serde_json::from_str`. If deserialization fails, the error includes
    /// a truncated excerpt of the raw response body for debugging.
    pub async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> EveryMapResult<T> {
        let start = Instant::now();
        let response = self.request(builder).await?;
        let body = response.text().await.map_err(EveryMapError::ClientError)?;

        if self.verbose {
            let snippet = truncate_str(&body, VERBOSE_BODY_LIMIT);
            eprintln!(
                "[VERBOSE] Response body ({:.0}ms, {} bytes):\n{}",
                start.elapsed().as_secs_f64() * 1000.0,
                body.len(),
                snippet
            );
        }

        serde_json::from_str::<T>(&body).map_err(|e| {
            let snippet = truncate_str(&body, ERROR_BODY_LIMIT);
            EveryMapError::provider(
                self.provider_name,
                "DESERIALIZATION_ERROR",
                format!("Failed to deserialize response: {}\nResponse body (first {} bytes): {}", e, ERROR_BODY_LIMIT, snippet),
            )
        })
    }

    /// Sends a POST request with a JSON body and deserializes the response.
    pub async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> EveryMapResult<T> {
        let builder = self
            .build_request(reqwest::Method::POST, url)
            .json(body);
        self.request_json(builder).await
    }
}

/// Redact API key values from a URL string for verbose output.
/// Replaces the value of common API key query params with `***`.
pub fn redact_api_key(url: &str) -> String {
    let mut result = url.to_string();
    for param in &["apiKey", "key", "access_token"] {
        let prefix = format!("{}=", param);
        if let Some(start) = result.find(&prefix) {
            let val_start = start + prefix.len();
            let val_end = result[val_start..]
                .find('&')
                .map(|i| val_start + i)
                .unwrap_or(result.len());
            result.replace_range(val_start..val_end, "***");
        }
    }
    result
}

/// Truncate a string to the given byte limit, appending info if truncated.
pub fn truncate_str(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .take_while(|(idx, _)| *idx < limit)
            .last()
            .map(|(idx, c)| idx + c.len_utf8())
            .unwrap_or(limit.min(s.len()));
        format!("{}...\n[truncated, {} bytes total]", &s[..end], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new("test".to_string(), "key".to_string()));
        let client = ProviderClient::new(auth, "test");
        assert_eq!(client.provider_name(), "test");
        assert!(!client.is_verbose());
    }

    #[test]
    fn test_provider_client_verbose() {
        let auth: Arc<dyn AuthProvider> = Arc::new(crate::auth::ApiKeyProvider::new("test".to_string(), "key".to_string()));
        let mut client = ProviderClient::new(auth, "test");
        client.set_verbose(true);
        assert!(client.is_verbose());
    }
}