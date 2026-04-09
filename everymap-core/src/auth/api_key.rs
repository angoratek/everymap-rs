use async_trait::async_trait;
use reqwest::RequestBuilder;
use crate::auth::provider::AuthProvider;
use crate::error::EveryMapResult;

/// Provider for API Key based authentication.
pub struct ApiKeyProvider {
    key: String,
    param_name: String,
}

impl ApiKeyProvider {
    pub fn new(key: String, param_name: String) -> Self {
        Self { key, param_name }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn apply(&self, request: RequestBuilder) -> EveryMapResult<RequestBuilder> {
        // Most map APIs use query parameters for API keys, but some use headers.
        // For now, we default to query parameters.
        Ok(request.query(&[(&self.param_name, &self.key)]))
    }
}
