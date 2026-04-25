use crate::auth::provider::AuthProvider;
use crate::error::EveryMapResult;
use async_trait::async_trait;
use reqwest::RequestBuilder;
use zeroize::Zeroize;

/// Provider for API Key based authentication.
///
/// Keys are zeroized on drop to prevent lingering sensitive data in memory.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ApiKeyProvider {
    key: String,
    #[zeroize(skip)]
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
        Ok(request.query(&[(&self.param_name, &self.key)]))
    }
}
