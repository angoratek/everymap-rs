use everymap_core::auth::AuthProvider;
use everymap_core::error::{EveryMapError, EveryMapResult};
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;

/// Maximum bytes of response body to display in verbose mode.
const VERBOSE_BODY_LIMIT: usize = 10_240;

/// The shared HTTP client for Google Maps APIs.
///
/// Handles authentication via API key injection. Each domain module
/// constructs its own base URL per the Google Maps API specification.
pub struct GoogleClient {
    http_client: Client,
    auth_provider: Arc<dyn AuthProvider>,
    verbose: bool,
}

impl GoogleClient {
    /// Creates a new `GoogleClient` with the given authentication provider.
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            http_client: Client::new(),
            auth_provider,
            verbose: false,
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
            eprintln!("[VERBOSE] {} {} — {} ({:.0}ms)", status.as_u16(), redact_api_key(&url), status.canonical_reason().unwrap_or("Unknown"), elapsed.as_secs_f64() * 1000.0);
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

    /// Sends a request and deserializes the JSON response into `T`.
    ///
    /// Reads the response body as text first, then deserializes with
    /// `serde_json::from_str`. If deserialization fails, the error includes
    /// the raw response body (truncated to 2KB) for easier debugging.
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
                    "google",
                    "DESERIALIZATION_ERROR",
                    format!("Failed to deserialize response: {}\nResponse body (first 2KB): {}", e, snippet),
                )
            })
    }
}

/// Redact API key values from a URL string for verbose output.
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

/// Truncate a string to the given byte limit, appending info if truncated.
fn truncate_str(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        s.to_string()
    } else {
        let end = s.char_indices()
            .take_while(|(idx, _)| *idx < limit)
            .last()
            .map(|(idx, c)| idx + c.len_utf8())
            .unwrap_or(limit.min(s.len()));
        format!("{}...\n[truncated, {} bytes total]", &s[..end], s.len())
    }
}