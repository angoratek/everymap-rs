//! Translation of structurally unusable TomTom payloads into the EveryMap
//! error model.
//!
//! A TomTom payload that decodes as JSON but lacks a field the projection
//! needs cannot produce a result the contract promises, so it is reported as
//! a provider failure with [`INVALID_RESPONSE_CODE`] instead of a plausible
//! empty result.

use everymap_core::error::EveryMapError;

/// Provider name carried by every TomTom error.
pub(crate) const TOMTOM_PROVIDER_NAME: &str = "tomtom";

/// Provider error code reported when a payload lacks a field the projection
/// needs.
pub const INVALID_RESPONSE_CODE: &str = "INVALID_RESPONSE";

/// Builds the provider error for a structurally unusable payload.
///
/// # Parameters
/// - `detail` — what the payload is missing or why it cannot be projected.
///
/// # Returns
/// An [`EveryMapError::ProviderError`] with [`INVALID_RESPONSE_CODE`].
pub(crate) fn invalid_response(detail: impl Into<String>) -> EveryMapError {
    EveryMapError::provider(TOMTOM_PROVIDER_NAME, INVALID_RESPONSE_CODE, detail)
}

#[cfg(test)]
mod tests {
    use super::invalid_response;
    use everymap_core::error::EveryMapError;

    #[test]
    fn an_invalid_response_carries_the_stable_code() {
        match invalid_response("missing shape") {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "INVALID_RESPONSE"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }
}
