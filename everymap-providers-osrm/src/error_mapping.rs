//! Translation of OSRM response envelopes into the unified error model.
//!
//! OSRM reports application failures in a JSON envelope instead of a bare HTTP
//! status:
//!
//! ```json
//! {"code": "NoRoute", "message": "Impossible route between points"}
//! ```
//!
//! The envelope may arrive with a success status or with a `4xx` status, so the
//! body is inspected before the HTTP status decides the outcome. A body that
//! is not valid JSON on a success response is a serialization failure; an
//! unsuccessful status without an OSRM envelope keeps the HTTP error.

use crate::client::OSRM_PROVIDER_NAME;
use everymap_core::error::{EveryMapError, EveryMapResult};
use serde_json::Value;

/// The `code` value of an OSRM response that carries a usable payload.
pub const OSRM_SUCCESS_CODE: &str = "Ok";

/// Error code reported when an OSRM payload is valid JSON but lacks a field
/// the route projection needs.
const OSRM_INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// Returns the provider error an OSRM `code` names, or `None` for a successful
/// or code-less payload.
///
/// # Parameters
/// - `payload` — a decoded OSRM response body.
///
/// # Returns
/// `Some` with a [`EveryMapError::ProviderError`] when `code` is present and
/// differs from [`OSRM_SUCCESS_CODE`]; the error carries the service code and
/// the service message verbatim (or the code when no message is present).
pub(crate) fn provider_error_from_payload(payload: &Value) -> Option<EveryMapError> {
    let code = payload.get("code").and_then(Value::as_str)?;
    if code == OSRM_SUCCESS_CODE {
        return None;
    }
    let message = payload
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or(code);
    Some(EveryMapError::provider(OSRM_PROVIDER_NAME, code, message))
}

/// Interprets the body of a successful OSRM response.
///
/// # Parameters
/// - `body` — the response body text.
///
/// # Returns
/// The decoded JSON payload when it carries no error envelope.
///
/// # Errors
/// - [`EveryMapError::SerializationError`] when the body is not valid JSON.
/// - [`EveryMapError::ProviderError`] when the body carries a `code` other
///   than [`OSRM_SUCCESS_CODE`].
pub(crate) fn interpret_success_body(body: &str) -> EveryMapResult<Value> {
    let payload: Value = serde_json::from_str(body)?;
    if let Some(provider_error) = provider_error_from_payload(&payload) {
        return Err(provider_error);
    }
    Ok(payload)
}

/// Extracts an OSRM provider error from the body of an unsuccessful response.
///
/// # Parameters
/// - `body` — the response body text.
///
/// # Returns
/// `Some` when the body is an OSRM error envelope; `None` when it is not JSON
/// or carries no error code.
pub(crate) fn provider_error_from_body(body: &str) -> Option<EveryMapError> {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|payload| provider_error_from_payload(&payload))
}

/// Builds the provider error for a structurally unusable OSRM payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it is unusable.
///
/// # Returns
/// A [`EveryMapError::ProviderError`] with the `INVALID_RESPONSE` code.
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(OSRM_PROVIDER_NAME, OSRM_INVALID_RESPONSE_CODE, detail)
}

#[cfg(test)]
mod tests {
    use super::{
        interpret_success_body, provider_error_from_body, provider_error_from_payload,
        OSRM_SUCCESS_CODE,
    };
    use everymap_core::error::EveryMapError;
    use serde_json::json;

    #[test]
    fn success_envelope_yields_the_payload() {
        let payload = interpret_success_body(r#"{"code":"Ok","routes":[]}"#).unwrap();
        assert_eq!(payload["code"], OSRM_SUCCESS_CODE);
    }

    #[test]
    fn provider_rejection_keeps_the_code_and_message_verbatim() {
        let error = provider_error_from_payload(&json!({
            "code": "NoRoute",
            "message": "Impossible route between points"
        }))
        .expect("a non-Ok code must surface");
        match error {
            EveryMapError::ProviderError {
                provider,
                code,
                message,
            } => {
                assert_eq!(provider, "osrm");
                assert_eq!(code, "NoRoute");
                assert_eq!(message, "Impossible route between points");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn a_code_without_a_message_keeps_the_code_as_the_message() {
        let error = provider_error_from_payload(&json!({"code": "NoSegment"}))
            .expect("a non-Ok code must surface");
        match error {
            EveryMapError::ProviderError { code, message, .. } => {
                assert_eq!(code, "NoSegment");
                assert_eq!(message, "NoSegment");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn a_success_body_without_an_envelope_passes_through() {
        assert!(
            provider_error_from_payload(&json!({"routes": []})).is_none(),
            "a code-less body is not an error envelope"
        );
    }

    #[test]
    fn malformed_successful_body_is_a_serialization_failure() {
        let error = interpret_success_body("not json").unwrap_err();
        assert!(matches!(error, EveryMapError::SerializationError { .. }));
    }

    #[test]
    fn a_non_json_error_body_is_not_a_provider_envelope() {
        assert!(provider_error_from_body("upstream failure").is_none());
    }
}
