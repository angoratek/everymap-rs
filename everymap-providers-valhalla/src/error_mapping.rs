//! Translation of Valhalla response envelopes into the unified error model.
//!
//! Valhalla reports a refusal as a non-success HTTP status whose JSON body
//! carries an `error_code` and a human-readable `error`:
//!
//! ```json
//! {"error_code": 440, "error": "No route found between origin and destination"}
//! ```
//!
//! A `4xx` body with that envelope becomes a provider error carrying the
//! service message verbatim; a `5xx` response stays an HTTP error, because the
//! server itself is unavailable rather than refusing the route. HTTP 429 is
//! reported as [`everymap_core::error::EveryMapError::RateLimited`] by the
//! shared client. Some deployments answer a refusal with a success status, so
//! a success body that carries `error` is refused as well.

use crate::client::VALHALLA_PROVIDER_NAME;
use everymap_core::error::EveryMapError;
use serde_json::Value;

/// Error code reported when a Valhalla payload is valid JSON but lacks a field
/// the route projection needs.
const VALHALLA_INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// Error code reported when an encoded limb shape cannot be decoded.
const VALHALLA_INVALID_POLYLINE_CODE: &str = "INVALID_POLYLINE";

/// Returns the provider error a Valhalla refusal names, or `None` when the
/// payload carries neither an `error` message nor an `error_code`.
///
/// # Parameters
/// - `payload` — a decoded Valhalla response body.
///
/// # Returns
/// `Some` with a [`EveryMapError::ProviderError`] carrying the service error
/// code (or `INVALID_RESPONSE` when only a message is present) and the service
/// message verbatim.
pub(crate) fn provider_error_from_payload(payload: &Value) -> Option<EveryMapError> {
    let error_message = payload.get("error").and_then(Value::as_str);
    let error_code = payload.get("error_code").map(|value| match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    });
    match (error_message, error_code) {
        (Some(message), Some(code)) => Some(EveryMapError::provider(
            VALHALLA_PROVIDER_NAME,
            code,
            message,
        )),
        (Some(message), None) => Some(EveryMapError::provider(
            VALHALLA_PROVIDER_NAME,
            VALHALLA_INVALID_RESPONSE_CODE,
            message,
        )),
        (None, Some(code)) => Some(EveryMapError::provider(
            VALHALLA_PROVIDER_NAME,
            code.clone(),
            format!("Valhalla error_code {code}"),
        )),
        (None, None) => None,
    }
}

/// Extracts a Valhalla provider error from the body of an unsuccessful
/// response.
///
/// # Parameters
/// - `body` — the response body text.
///
/// # Returns
/// `Some` when the body is a Valhalla error object; `None` when it is not JSON
/// or carries no Valhalla error field.
pub(crate) fn provider_error_from_body(body: &str) -> Option<EveryMapError> {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|payload| provider_error_from_payload(&payload))
}

/// Builds the provider error for a structurally unusable Valhalla payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it is unusable.
///
/// # Returns
/// A [`EveryMapError::ProviderError`] with the `INVALID_RESPONSE` code.
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(
        VALHALLA_PROVIDER_NAME,
        VALHALLA_INVALID_RESPONSE_CODE,
        detail,
    )
}

/// Builds the provider error for an encoded shape that cannot be decoded.
///
/// # Parameters
/// - `detail` — why the shape is unusable.
///
/// # Returns
/// A [`EveryMapError::ProviderError`] with the `INVALID_POLYLINE` code.
pub(crate) fn invalid_polyline(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(
        VALHALLA_PROVIDER_NAME,
        VALHALLA_INVALID_POLYLINE_CODE,
        detail,
    )
}

#[cfg(test)]
mod tests {
    use super::{provider_error_from_body, provider_error_from_payload};
    use everymap_core::error::EveryMapError;
    use serde_json::json;

    #[test]
    fn valhalla_error_message_is_surfaced_verbatim() {
        let error = provider_error_from_payload(&json!({
            "error_code": 440,
            "error": "No route found between origin and destination",
            "status_code": 400
        }))
        .expect("a refusal must surface");
        match error {
            EveryMapError::ProviderError {
                provider,
                code,
                message,
            } => {
                assert_eq!(provider, "valhalla");
                assert_eq!(code, "440");
                assert_eq!(message, "No route found between origin and destination");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn error_code_without_a_message_keeps_the_code() {
        let error = provider_error_from_payload(&json!({"error_code": 120}))
            .expect("an error code must surface");
        match error {
            EveryMapError::ProviderError { code, message, .. } => {
                assert_eq!(code, "120");
                assert_eq!(message, "Valhalla error_code 120");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn a_message_without_a_code_uses_the_invalid_response_code() {
        let error = provider_error_from_payload(&json!({"error": "refused"}))
            .expect("an error message must surface");
        match error {
            EveryMapError::ProviderError { code, message, .. } => {
                assert_eq!(code, "INVALID_RESPONSE");
                assert_eq!(message, "refused");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn a_body_without_an_error_field_is_not_a_refusal() {
        assert!(provider_error_from_payload(&json!({"trip": {"status": 0}})).is_none());
        assert!(provider_error_from_body("upstream failure").is_none());
    }
}
