use thiserror::Error;

/// Unified error type for all EveryMap operations.
///
/// Provides structured error information across HTTP, authentication,
/// provider-specific, rate limiting, and validation scenarios.
#[derive(Error, Debug)]
pub enum EveryMapError {
    /// HTTP request failed with a status code.
    #[error("HTTP error {status}: {message}")]
    HttpError {
        status: u16,
        message: String,
        body: Option<String>,
    },

    /// Authentication failed (invalid API key, expired token, etc.).
    #[error("Authentication failed: {message}")]
    AuthError {
        message: String,
        provider: String,
    },

    /// Provider-specific error (e.g., HERE returned an error response).
    #[error("{provider} error: {code} - {message}")]
    ProviderError {
        provider: String,
        code: String,
        message: String,
    },

    /// Rate limited by the provider.
    #[error("Rate limited by {provider}, retry after {retry_after_secs:?}s")]
    RateLimited {
        provider: String,
        retry_after_secs: Option<u64>,
    },

    /// Request validation failed (invalid coordinates, missing required fields, etc.).
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Failed to deserialize a response body.
    #[error("Failed to deserialize response: {source}")]
    SerializationError {
        source: serde_json::Error,
    },

    /// An error occurred in the underlying HTTP client.
    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),

    /// An unknown or unexpected error occurred.
    #[error("Unknown error occurred")]
    Unknown,
}

impl EveryMapError {
    /// Create an HTTP error from a status code and message.
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::HttpError {
            status,
            message: message.into(),
            body: None,
        }
    }

    /// Create an HTTP error with a response body.
    pub fn http_with_body(status: u16, message: impl Into<String>, body: impl Into<String>) -> Self {
        Self::HttpError {
            status,
            message: message.into(),
            body: Some(body.into()),
        }
    }

    /// Create a provider error.
    pub fn provider(provider: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProviderError {
            provider: provider.into(),
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create an auth error.
    pub fn auth(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AuthError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a rate limit error.
    pub fn rate_limited(provider: impl Into<String>, retry_after_secs: Option<u64>) -> Self {
        Self::RateLimited {
            provider: provider.into(),
            retry_after_secs,
        }
    }

    /// Check if this is an HTTP error with the given status code.
    pub fn is_status(&self, status: u16) -> bool {
        match self {
            Self::HttpError { status: s, .. } => *s == status,
            _ => false,
        }
    }

    /// Check if this is a rate limit error (HTTP 429).
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. }) || self.is_status(429)
    }

    /// Check if this is an authentication error (HTTP 401 or 403).
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::AuthError { .. }) || self.is_status(401) || self.is_status(403)
    }
}

// Maintain backward compatibility: `From<reqwest::Error>` now maps to `ClientError`
// rather than `HttpError` since we have richer HTTP error handling now.
// `SerializationError` keeps its `from` impl for serde_json::Error.

impl From<serde_json::Error> for EveryMapError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { source: err }
    }
}

pub type EveryMapResult<T> = Result<T, EveryMapError>;