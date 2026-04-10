use crate::error::EveryMapResult;
use async_trait::async_trait;

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