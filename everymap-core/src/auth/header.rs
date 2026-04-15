use async_trait::async_trait;
use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::RequestBuilder;
use crate::auth::provider::AuthProvider;
use crate::error::EveryMapResult;

/// Provider for header-based authentication (e.g., Radar's `Authorization` header).
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

#[async_trait]
impl AuthProvider for HeaderAuthProvider {
    async fn apply(&self, request: RequestBuilder) -> EveryMapResult<RequestBuilder> {
        let header_value = HeaderValue::from_str(&self.value)
            .map_err(|e| crate::error::EveryMapError::AuthError {
                provider: "unknown".to_string(),
                message: format!("Invalid auth header value: {}", e),
            })?;
        Ok(request.header(AUTHORIZATION, header_value))
    }
}