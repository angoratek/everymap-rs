use everymap_core::auth::AuthProvider;
use everymap_core::client::HttpClient;
use everymap_core::error::{EveryMapError, EveryMapResult};
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;

/// Maximum bytes of response body to display in verbose mode.
const VERBOSE_BODY_LIMIT: usize = 10_240;

/// The shared HTTP client for HERE Technologies APIs.
///
/// This client is a thin wrapper around `reqwest::Client` that handles
/// authentication. Each domain module constructs its own base URL per the
/// HERE API specification.
///
/// For testing, use `with_http_client()` to inject a custom `HttpClient`
/// implementation. For production, use `new()` with the default client.
pub struct HereClient {
    http_client: Client,
    auth_provider: Arc<dyn AuthProvider>,
    verbose: bool,
}

impl HereClient {
    /// Creates a new `HereClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            http_client: Client::new(),
            auth_provider,
            verbose: false,
        }
    }

    /// Creates a `HereClient` with a custom `reqwest::Client` configuration.
    pub fn with_client_builder(
        builder: reqwest::ClientBuilder,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> EveryMapResult<Self> {
        Ok(Self {
            http_client: builder.build()?,
            auth_provider,
            verbose: false,
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
            eprintln!("[VERBOSE] {} {} — {} ({:.0}ms)", response.status().as_u16(), redact_api_key(&url), status.canonical_reason().unwrap_or("Unknown"), elapsed.as_secs_f64() * 1000.0);
        }

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

            if self.verbose {
                let snippet = truncate_str(&body, VERBOSE_BODY_LIMIT);
                eprintln!("[VERBOSE] Error response body:\n{}", snippet);
            }

            if status_code == 429 {
                Err(EveryMapError::rate_limited("here", retry_after))
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
    /// This reads the response body as text first, then deserializes with
    /// `serde_json::from_str`. If deserialization fails, the error includes
    /// the raw response body (truncated to 2KB) for easier debugging.
    /// This avoids the opaque "error decoding response body" message from
    /// `reqwest`'s built-in `.json()`.
    pub async fn request_json<T: serde::de::DeserializeOwned>(&self, builder: reqwest::RequestBuilder) -> EveryMapResult<T> {
        let start = Instant::now();
        let response = self.request(builder).await?;
        let body = response.text().await
            .map_err(EveryMapError::ClientError)?;

        if self.verbose {
            let snippet = truncate_str(&body, VERBOSE_BODY_LIMIT);
            eprintln!("[VERBOSE] Response body ({:.0}ms, {} bytes):\n{}", start.elapsed().as_secs_f64() * 1000.0, body.len(), snippet);
        }

        serde_json::from_str::<T>(&body)
            .map_err(|e| {
                let snippet = if body.len() > 2048 { &body[..2048] } else { &body };
                EveryMapError::provider(
                    "here",
                    "DESERIALIZATION_ERROR",
                    format!("Failed to deserialize response: {}\nResponse body (first 2KB): {}", e, snippet),
                )
            })
    }
}

/// Sends a request using a custom `HttpClient` implementation, applying auth first.
/// This is useful for testing with mock HTTP clients.
pub async fn send_with_auth(
    http_client: &dyn HttpClient,
    auth_provider: &Arc<dyn AuthProvider>,
    builder: reqwest::RequestBuilder,
) -> EveryMapResult<reqwest::Response> {
    let builder = auth_provider.apply(builder).await?;
    http_client.send(builder).await
}

/// Redact API key values from a URL string for verbose output.
/// Replaces the value of `apiKey`, `key`, and `access_token` params with `***`.
fn redact_api_key(url: &str) -> String {
    let mut result = url.to_string();
    for param in &["apiKey", "key", "access_token"] {
        let prefix = format!("{}=", param);
        if let Some(start) = result.find(&prefix) {
            let val_start = start + prefix.len();
            let val_end = result[val_start..].find('&').map(|i| val_start + i).unwrap_or(result.len());
            result.replace_range(val_start..val_end, "***");
        }
    }
    result
}

/// Truncate a string to the given byte limit, appending "..." if truncated.
fn truncate_str(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        s.to_string()
    } else {
        // Find a valid char boundary near the limit
        let end = s.char_indices()
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
        let redacted = redact_api_key(url);
        assert_eq!(redacted, "https://router.hereapi.com/v8/routes?apiKey=***&origin=52.5,13.3");
    }

    #[test]
    fn test_redact_api_key_google() {
        let url = "https://maps.googleapis.com/maps/api/geocode/json?key=secret123&address=Berlin";
        let redacted = redact_api_key(url);
        assert_eq!(redacted, "https://maps.googleapis.com/maps/api/geocode/json?key=***&address=Berlin");
    }

    #[test]
    fn test_redact_api_key_mapbox() {
        let url = "https://api.mapbox.com/geocode/v6/forward?q=test&access_token=pk.secret123";
        let redacted = redact_api_key(url);
        assert_eq!(redacted, "https://api.mapbox.com/geocode/v6/forward?q=test&access_token=***");
    }

    #[test]
    fn test_redact_no_key() {
        let url = "https://example.com/api?foo=bar";
        let redacted = redact_api_key(url);
        assert_eq!(redacted, url);
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
}