use crate::error::EveryMapResult;
use async_trait::async_trait;
use reqwest::RequestBuilder;

/// Trait for applying authentication to outgoing HTTP requests.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Modifies the request builder to include necessary authentication credentials.
    async fn apply(&self, request: RequestBuilder) -> EveryMapResult<RequestBuilder>;
}
