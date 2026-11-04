use crate::auth::provider::AuthProvider;
use crate::error::{EveryMapError, EveryMapResult};
use crate::http::HttpRequest;
use async_trait::async_trait;
use zeroize::Zeroize;

/// Header used by [`HeaderAuthProvider`].
pub(crate) const AUTHORIZATION_HEADER: &str = "authorization";

/// Provider for header-based authentication (e.g., Radar's `Authorization` header).
///
/// The key value is zeroized on drop to prevent lingering sensitive data in memory.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct HeaderAuthProvider {
    value: String,
}

impl HeaderAuthProvider {
    /// Create a new header auth provider with the raw header value.
    ///
    /// The value is set as the `Authorization` header. For Radar, this is
    /// the API key directly (e.g., `prj_live_pk_...`).
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// Whether a value can be sent as an HTTP header value.
///
/// Rejects control characters (except horizontal tab) and the empty value.
pub(crate) fn is_valid_header_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte == b'\t' || (0x20..=0x7e).contains(&byte) || byte >= 0x80)
}

#[async_trait]
impl AuthProvider for HeaderAuthProvider {
    async fn apply(&self, request: HttpRequest) -> EveryMapResult<HttpRequest> {
        if !is_valid_header_value(&self.value) {
            return Err(EveryMapError::auth(
                "unknown",
                "Invalid auth header value: contains control characters or is empty",
            ));
        }
        Ok(request.header(AUTHORIZATION_HEADER, self.value.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_visible_ascii_and_tab() {
        assert!(is_valid_header_value("prj_live_pk_123"));
        assert!(is_valid_header_value("Bearer abc\tdef"));
    }

    #[test]
    fn rejects_empty_and_control_characters() {
        assert!(!is_valid_header_value(""));
        assert!(!is_valid_header_value("line\nbreak"));
        assert!(!is_valid_header_value("carriage\rreturn"));
        assert!(!is_valid_header_value("nul\0byte"));
    }
}
