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
    AuthError { message: String, provider: String },

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
    SerializationError { source: serde_json::Error },

    /// An error occurred in the underlying HTTP client.
    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),

    /// The requested domain is not supported by this provider.
    #[error("{provider} does not support {domain}")]
    UnsupportedDomain { provider: String, domain: String },

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
    pub fn http_with_body(
        status: u16,
        message: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self::HttpError {
            status,
            message: message.into(),
            body: Some(body.into()),
        }
    }

    /// Create a provider error.
    pub fn provider(
        provider: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
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

// Maintain backward compatibility: `From<reqwest::Error>` now maps to `ClientError`
// rather than `HttpError` since we have richer HTTP error handling now.
// `SerializationError` keeps its `from` impl for serde_json::Error.

impl From<serde_json::Error> for EveryMapError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { source: err }
    }
}

pub type EveryMapResult<T> = Result<T, EveryMapError>;

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
            EveryMapError::ProviderError {
                provider,
                code,
                message,
            } => {
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

    // --- All error variants construction ---

    #[test]
    fn test_http_error_construction_basic() {
        let err = EveryMapError::http(500, "Server Error");
        assert!(err.is_status(500));
        assert!(!err.is_status(200));
    }

    #[test]
    fn test_http_error_with_body_construction() {
        let err = EveryMapError::http_with_body(502, "Bad Gateway", "upstream timeout");
        assert!(err.is_status(502));
    }

    #[test]
    fn test_auth_error_construction() {
        let err = EveryMapError::auth("google", "API key expired");
        assert!(err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_provider_error_construction() {
        let err = EveryMapError::provider("tomtom", "E404", "Not found");
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_rate_limited_construction() {
        let err = EveryMapError::rate_limited("here", Some(30));
        assert!(err.is_rate_limited());
        assert!(!err.is_auth_error());
    }

    #[test]
    fn test_rate_limited_without_retry_after() {
        let err = EveryMapError::rate_limited("mapbox", None);
        assert!(err.is_rate_limited());
    }

    #[test]
    fn test_validation_error_construction() {
        let err = EveryMapError::ValidationError("Coordinates out of range".to_string());
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_serialization_error_construction() {
        let json_str = "not valid json{{{";
        let serde_err: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let err = EveryMapError::from(serde_err.unwrap_err());
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_unsupported_domain_construction() {
        let err = EveryMapError::unsupported_domain("radar", "imaging");
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    #[test]
    fn test_unknown_error_construction() {
        let err = EveryMapError::Unknown;
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    // --- Display formatting for each variant ---

    #[test]
    fn test_display_http_error() {
        let err = EveryMapError::http(404, "Not Found");
        let msg = format!("{}", err);
        assert!(msg.contains("404"));
        assert!(msg.contains("Not Found"));
    }

    #[test]
    fn test_display_auth_error() {
        let err = EveryMapError::auth("here", "Invalid key");
        let msg = format!("{}", err);
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("Invalid key"));
    }

    #[test]
    fn test_display_provider_error() {
        let err = EveryMapError::provider("here", "E400", "Bad request");
        let msg = format!("{}", err);
        assert!(msg.contains("here"));
        assert!(msg.contains("E400"));
        assert!(msg.contains("Bad request"));
    }

    #[test]
    fn test_display_rate_limited() {
        let err = EveryMapError::rate_limited("google", Some(60));
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limited"));
        assert!(msg.contains("google"));
        assert!(msg.contains("60"));
    }

    #[test]
    fn test_display_rate_limited_no_retry() {
        let err = EveryMapError::rate_limited("google", None);
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limited"));
    }

    #[test]
    fn test_display_validation_error() {
        let err = EveryMapError::ValidationError("bad input".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Validation error"));
        assert!(msg.contains("bad input"));
    }

    #[test]
    fn test_display_serialization_error() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let err = EveryMapError::from(serde_err);
        let msg = format!("{}", err);
        assert!(msg.contains("Failed to deserialize"));
    }

    #[test]
    fn test_display_unsupported_domain() {
        let err = EveryMapError::unsupported_domain("google", "traffic");
        let msg = format!("{}", err);
        assert!(msg.contains("google"));
        assert!(msg.contains("traffic"));
    }

    #[test]
    fn test_display_unknown() {
        let err = EveryMapError::Unknown;
        let msg = format!("{}", err);
        assert!(msg.contains("Unknown"));
    }

    // --- is_status with various codes ---

    #[test]
    fn test_is_status_200() {
        let err = EveryMapError::http(200, "OK");
        assert!(err.is_status(200));
        assert!(!err.is_status(201));
    }

    #[test]
    fn test_is_status_403() {
        let err = EveryMapError::http(403, "Forbidden");
        assert!(err.is_status(403));
    }

    #[test]
    fn test_is_status_non_http_error_returns_false() {
        let err = EveryMapError::ValidationError("test".to_string());
        assert!(!err.is_status(400));
    }

    // --- is_rate_limited with RateLimited and HttpError 429 ---

    #[test]
    fn test_is_rate_limited_rate_limited_variant() {
        let err = EveryMapError::rate_limited("here", Some(60));
        assert!(err.is_rate_limited());
    }

    #[test]
    fn test_is_rate_limited_http_429() {
        let err = EveryMapError::http(429, "Too Many Requests");
        assert!(err.is_rate_limited());
    }

    #[test]
    fn test_is_rate_limited_http_other_code() {
        let err = EveryMapError::http(500, "Server Error");
        assert!(!err.is_rate_limited());
    }

    // --- is_auth_error with AuthError and HttpError 401/403 ---

    #[test]
    fn test_is_auth_error_auth_variant() {
        let err = EveryMapError::auth("here", "Invalid key");
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_is_auth_error_http_401() {
        let err = EveryMapError::http(401, "Unauthorized");
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_is_auth_error_http_403() {
        let err = EveryMapError::http(403, "Forbidden");
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_is_auth_error_http_other_code() {
        let err = EveryMapError::http(500, "Server Error");
        assert!(!err.is_auth_error());
    }

    // --- From<serde_json::Error> conversion ---

    #[test]
    fn test_from_serde_json_error() {
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: EveryMapError = serde_err.into();
        match err {
            EveryMapError::SerializationError { .. } => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_from_serde_json_error_is_not_auth() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let err: EveryMapError = serde_err.into();
        assert!(!err.is_auth_error());
        assert!(!err.is_rate_limited());
    }

    // --- HttpError body field ---

    #[test]
    fn test_http_error_with_body_contains_body() {
        match EveryMapError::http_with_body(500, "Error", "detailed body") {
            EveryMapError::HttpError {
                status,
                message,
                body,
            } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Error");
                assert_eq!(body, Some("detailed body".to_string()));
            }
            _ => panic!("Expected HttpError"),
        }
    }

    #[test]
    fn test_http_error_without_body() {
        match EveryMapError::http(404, "Not Found") {
            EveryMapError::HttpError {
                status,
                message,
                body,
            } => {
                assert_eq!(status, 404);
                assert_eq!(message, "Not Found");
                assert_eq!(body, None);
            }
            _ => panic!("Expected HttpError"),
        }
    }

    // --- RateLimited retry_after_secs field ---

    #[test]
    fn test_rate_limited_retry_after_some() {
        match EveryMapError::rate_limited("here", Some(120)) {
            EveryMapError::RateLimited {
                provider,
                retry_after_secs,
            } => {
                assert_eq!(provider, "here");
                assert_eq!(retry_after_secs, Some(120));
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[test]
    fn test_rate_limited_retry_after_none() {
        match EveryMapError::rate_limited("here", None) {
            EveryMapError::RateLimited {
                retry_after_secs, ..
            } => {
                assert_eq!(retry_after_secs, None);
            }
            _ => panic!("Expected RateLimited"),
        }
    }
}
