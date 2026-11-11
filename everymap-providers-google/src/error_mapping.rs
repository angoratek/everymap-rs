//! Translation of Google Maps application-level failures into the EveryMap
//! error model.
//!
//! The classic Google Maps web services answer a successful HTTP status with
//! an in-body `status` field. `REQUEST_DENIED` means the credential is not
//! usable, the two quota statuses mean the caller must slow down, and every
//! other non-success status is a provider failure. The Routes API v2 reports
//! failures through the HTTP status instead, which the shared client already
//! maps.
//!
//! A payload that is valid JSON but lacks a field the projection needs is a
//! structurally unusable response, reported with [`INVALID_RESPONSE_CODE`].

use everymap_core::error::EveryMapError;

/// Provider name carried by every Google error.
pub(crate) const GOOGLE_PROVIDER_NAME: &str = "google";

/// Provider error code reported when a payload lacks a field the projection
/// needs.
pub const INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// Google status reported when the credential is rejected.
const STATUS_REQUEST_DENIED: &str = "REQUEST_DENIED";

/// Google status reported when the per-second quota is exhausted.
const STATUS_OVER_QUERY_LIMIT: &str = "OVER_QUERY_LIMIT";

/// Google status reported when the daily quota is exhausted.
const STATUS_OVER_DAILY_LIMIT: &str = "OVER_DAILY_LIMIT";

/// Builds the provider error for a structurally unusable payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it cannot be projected.
///
/// # Returns
/// An [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`].
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(GOOGLE_PROVIDER_NAME, INVALID_RESPONSE_CODE, detail)
}

/// Maps a non-success Google `status` value onto the shared error categories.
///
/// # Parameters
/// - `status` — the in-body `status` string.
/// - `message` — the optional in-body `error_message`.
///
/// # Returns
/// - [`EveryMapError::AuthError`] for `REQUEST_DENIED`.
/// - [`EveryMapError::RateLimited`] for the two quota statuses.
/// - [`EveryMapError::ProviderError`] for every other status, carrying the
///   status verbatim as its code.
pub(crate) fn status_error(status: &str, message: Option<&str>) -> EveryMapError {
    let message = message
        .map(str::to_string)
        .unwrap_or_else(|| format!("Google rejected the request with status {status}"));
    match status {
        STATUS_REQUEST_DENIED => EveryMapError::AuthError {
            message,
            provider: GOOGLE_PROVIDER_NAME.to_string(),
        },
        STATUS_OVER_QUERY_LIMIT | STATUS_OVER_DAILY_LIMIT => EveryMapError::RateLimited {
            provider: GOOGLE_PROVIDER_NAME.to_string(),
            retry_after_secs: None,
        },
        _ => EveryMapError::provider(GOOGLE_PROVIDER_NAME, status, message),
    }
}

#[cfg(test)]
mod tests {
    use super::{status_error, GOOGLE_PROVIDER_NAME};
    use everymap_core::error::EveryMapError;

    #[test]
    fn request_denied_becomes_an_auth_error() {
        match status_error("REQUEST_DENIED", Some("The provided API key is invalid.")) {
            EveryMapError::AuthError { provider, message } => {
                assert_eq!(provider, GOOGLE_PROVIDER_NAME);
                assert!(message.contains("invalid"));
            }
            other => panic!("expected an auth error, got {other:?}"),
        }
    }

    #[test]
    fn quota_statuses_become_rate_limited() {
        for status in ["OVER_QUERY_LIMIT", "OVER_DAILY_LIMIT"] {
            assert!(matches!(
                status_error(status, None),
                EveryMapError::RateLimited {
                    provider,
                    retry_after_secs: None,
                } if provider == GOOGLE_PROVIDER_NAME
            ));
        }
    }

    #[test]
    fn another_status_keeps_the_provider_code() {
        match status_error("INVALID_REQUEST", None) {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "INVALID_REQUEST"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }
}
