//! Translation of Radar application-level failures into the EveryMap error
//! model.
//!
//! Radar reports application failures in the response envelope's `meta.code`
//! while the HTTP status stays successful. A rejected credential is an
//! authentication failure, an exhausted quota is a rate limit, and every
//! other non-200 code is a provider failure.
//!
//! A payload that is valid JSON but lacks a field the projection needs is a
//! structurally unusable response, reported with [`INVALID_RESPONSE_CODE`].

use everymap_core::error::EveryMapError;

/// Provider name carried by every Radar error.
pub(crate) const RADAR_PROVIDER_NAME: &str = "radar";

/// Provider error code reported when a payload lacks a field the projection
/// needs.
pub const INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// The Radar status code of a successful request.
pub(crate) const STATUS_OK: u16 = 200;

/// The Radar status code of a rejected credential.
const STATUS_UNAUTHORIZED: u16 = 401;

/// The Radar status code of a credential without permission.
const STATUS_FORBIDDEN: u16 = 403;

/// The Radar status code of an exhausted quota.
const STATUS_TOO_MANY_REQUESTS: u16 = 429;

/// Builds the provider error for a structurally unusable payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it cannot be projected.
///
/// # Returns
/// An [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`].
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(RADAR_PROVIDER_NAME, INVALID_RESPONSE_CODE, detail)
}

/// Maps a non-success Radar `meta.code` onto the shared error categories.
///
/// # Parameters
/// - `status_code` — the numeric code from the response envelope.
///
/// # Returns
/// - [`EveryMapError::AuthError`] for 401 and 403.
/// - [`EveryMapError::RateLimited`] for 429.
/// - [`EveryMapError::ProviderError`] for every other code, carrying the code
///   verbatim.
pub(crate) fn status_error(status_code: u16) -> EveryMapError {
    match status_code {
        STATUS_UNAUTHORIZED | STATUS_FORBIDDEN => EveryMapError::AuthError {
            message: format!("Radar rejected the credential with status {status_code}"),
            provider: RADAR_PROVIDER_NAME.to_string(),
        },
        STATUS_TOO_MANY_REQUESTS => EveryMapError::RateLimited {
            provider: RADAR_PROVIDER_NAME.to_string(),
            retry_after_secs: None,
        },
        _ => EveryMapError::provider(
            RADAR_PROVIDER_NAME,
            status_code.to_string(),
            format!("Radar rejected the request with status {status_code}"),
        ),
    }
}

/// Returns an error unless the envelope reports [`STATUS_OK`].
///
/// # Parameters
/// - `status_code` — the numeric code from the response envelope.
///
/// # Returns
/// `Ok(())` when the code is [`STATUS_OK`]; otherwise the error
/// [`status_error`] reports.
pub(crate) fn ensure_success(status_code: u16) -> everymap_core::error::EveryMapResult<()> {
    if status_code == STATUS_OK {
        Ok(())
    } else {
        Err(status_error(status_code))
    }
}

#[cfg(test)]
mod tests {
    use super::{ensure_success, status_error, RADAR_PROVIDER_NAME};
    use everymap_core::error::EveryMapError;

    #[test]
    fn credential_statuses_become_auth_errors() {
        for status_code in [401, 403] {
            match status_error(status_code) {
                EveryMapError::AuthError { provider, .. } => {
                    assert_eq!(provider, RADAR_PROVIDER_NAME);
                }
                other => panic!("expected an auth error, got {other:?}"),
            }
        }
    }

    #[test]
    fn quota_status_becomes_rate_limited() {
        assert!(matches!(
            status_error(429),
            EveryMapError::RateLimited { provider, .. } if provider == RADAR_PROVIDER_NAME
        ));
    }

    #[test]
    fn another_status_keeps_the_provider_code() {
        match status_error(422) {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "422"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn a_success_envelope_is_accepted() {
        assert!(ensure_success(200).is_ok());
        assert!(ensure_success(500).is_err());
    }
}
