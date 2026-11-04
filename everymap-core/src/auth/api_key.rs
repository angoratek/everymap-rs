use crate::auth::provider::AuthProvider;
use crate::error::EveryMapResult;
use crate::http::HttpRequest;
use async_trait::async_trait;
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
    /// Create a provider that adds the key as the named query parameter.
    pub fn new(key: String, param_name: String) -> Self {
        Self { key, param_name }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn apply(&self, request: HttpRequest) -> EveryMapResult<HttpRequest> {
        Ok(request.query_param(self.param_name.clone(), self.key.clone()))
    }
}
