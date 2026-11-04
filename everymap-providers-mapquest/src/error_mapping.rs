//! Translation of MapQuest application-level failures into the EveryMap error
//! model.
//!
//! MapQuest reports two distinct failure channels:
//!
//! 1. Transport-level failures carried by the HTTP status. The shared
//!    [`everymap_core::client::ProviderClient`] already maps those to
//!    [`everymap_core::error::EveryMapError::HttpError`] or
//!    [`everymap_core::error::EveryMapError::RateLimited`].
//! 2. Application-level failures where the service answers with a success
//!    status but a non-zero `info.statuscode` and human-readable
//!    `info.messages`. Those become
//!    [`everymap_core::error::EveryMapError::ProviderError`] here.
//!
//! The Search Ahead prediction endpoint carries no `info` envelope, so an
//! absent envelope is accepted. A payload that carries an envelope without a
//! numeric status code is structurally unusable and is reported as a provider
//! error with a stable code.

use crate::client::MAPQUEST_PROVIDER_NAME;
use everymap_core::error::{EveryMapError, EveryMapResult};
use serde_json::Value;

/// Provider error code reported when a payload lacks a field the projection
/// needs or carries a malformed MapQuest envelope.
pub const INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// Builds the provider error for a structurally unusable payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it cannot be projected.
///
/// # Returns
/// A [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`] and the
/// given detail.
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(MAPQUEST_PROVIDER_NAME, INVALID_RESPONSE_CODE, detail)
}

/// Validates a MapQuest `info` envelope, tolerating its absence.
///
/// # Parameters
/// - `payload` — a decoded MapQuest response body.
///
/// # Returns
/// `Ok(())` when the payload carries no envelope, or when the envelope reports
/// `statuscode` 0.
///
/// # Errors
/// - [`EveryMapError::ProviderError`] with the service status code when the
///   envelope reports a non-zero `statuscode`.
/// - [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`] when an
///   envelope is present but has no numeric `statuscode`.
pub(crate) fn validate_info_envelope_if_present(payload: &Value) -> EveryMapResult<()> {
    if payload.get("info").is_none() {
        return Ok(());
    }
    validate_info_envelope(payload)
}

/// Validates the MapQuest `info` envelope of a JSON payload.
///
/// # Parameters
/// - `payload` — a decoded MapQuest response body that carries `info`.
///
/// # Returns
/// `Ok(())` when `info.statuscode` is 0.
///
/// # Errors
/// - [`EveryMapError::ProviderError`] with the service status code as its code
///   and the joined `info.messages` as its message when `statuscode` is
///   non-zero.
/// - [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`] when the
///   envelope is absent or carries no numeric `statuscode`.
pub(crate) fn validate_info_envelope(payload: &Value) -> EveryMapResult<()> {
    let info = payload
        .get("info")
        .ok_or_else(|| invalid_response("payload lacks the MapQuest info envelope"))?;
    let status_code = info
        .get("statuscode")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_response("info envelope lacks a numeric statuscode"))?;
    if status_code == 0 {
        return Ok(());
    }
    let message = info
        .get("messages")
        .and_then(Value::as_array)
        .map(|messages| {
            messages
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<&str>>()
                .join("; ")
        })
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| "MapQuest rejected the request".to_string());
    Err(EveryMapError::provider(
        MAPQUEST_PROVIDER_NAME,
        status_code.to_string(),
        message,
    ))
}

#[cfg(test)]
mod tests {
    use super::{invalid_response, validate_info_envelope, validate_info_envelope_if_present};
    use crate::client::MAPQUEST_PROVIDER_NAME;
    use everymap_core::error::EveryMapError;
    use serde_json::json;

    #[test]
    fn a_success_envelope_is_accepted() {
        let payload = json!({"info": {"statuscode": 0, "messages": []}});
        assert!(validate_info_envelope(&payload).is_ok());
    }

    #[test]
    fn an_absent_envelope_is_accepted_when_tolerated() {
        assert!(validate_info_envelope_if_present(&json!({"results": []})).is_ok());
    }

    #[test]
    fn a_non_zero_status_code_becomes_a_provider_error() {
        let payload = json!({
            "info": {
                "statuscode": 500,
                "messages": ["Illegal argument from request: Invalid LatLng specified.[0]"]
            }
        });
        match validate_info_envelope(&payload) {
            Err(EveryMapError::ProviderError {
                provider,
                code,
                message,
            }) => {
                assert_eq!(provider, MAPQUEST_PROVIDER_NAME);
                assert_eq!(code, "500");
                assert!(message.contains("Invalid LatLng"));
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn an_envelope_without_a_status_code_is_invalid() {
        assert!(validate_info_envelope(&json!({"info": {}})).is_err());
        assert!(validate_info_envelope_if_present(&json!({"info": {}})).is_err());
    }

    #[test]
    fn an_invalid_response_carries_the_stable_code() {
        match invalid_response("missing shape") {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "INVALID_RESPONSE"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }
}
