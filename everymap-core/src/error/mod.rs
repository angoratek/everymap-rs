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

    /// The requested domain is not supported by this provider.
    #[error("{provider} does not support {domain}")]
    UnsupportedDomain {
        provider: String,
        domain: String,
    },

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

    /// Create an unsupported domain error.
    pub fn unsupported_domain(provider: impl Into<String>, domain: impl Into<String>) -> Self {
        Self::UnsupportedDomain {
            provider: provider.into(),
            domain: domain.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_error_construction() {
        let err = EveryMapError::http(404, "Not Found");
        assert!(err.is_status(404));
        assert!(!err.is_status(200));
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_http_error_with_body() {
        let err = EveryMapError::http_with_body(500, "Internal Error", "details");
        assert!(err.is_status(500));
    }

    #[test]
    fn test_auth_error() {
        let err = EveryMapError::auth("here", "Invalid API key");
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_rate_limited_error() {
        let err = EveryMapError::rate_limited("here", Some(60));
        assert!(err.is_rate_limited());
    }

    #[test]
    fn test_unsupported_domain_error() {
        let err = EveryMapError::unsupported_domain("google", "traffic");
        match &err {
            EveryMapError::UnsupportedDomain { provider, domain } => {
                assert_eq!(provider, "google");
                assert_eq!(domain, "traffic");
            }
            _ => panic!("Expected UnsupportedDomain error"),
        }
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_provider_error() {
        let err = EveryMapError::provider("here", "E400", "Bad request");
        match &err {
            EveryMapError::ProviderError { provider, code, message } => {
                assert_eq!(provider, "here");
                assert_eq!(code, "E400");
                assert_eq!(message, "Bad request");
            }
            _ => panic!("Expected ProviderError"),
        }
    }

    #[test]
    fn test_status_401_is_auth_error() {
        let err = EveryMapError::http(401, "Unauthorized");
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_status_429_is_rate_limited() {
        let err = EveryMapError::http(429, "Too Many Requests");
        assert!(err.is_rate_limited());
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