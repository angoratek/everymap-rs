use crate::error::EveryMapResult;
use crate::http::HttpRequest;
use async_trait::async_trait;

/// Trait for applying authentication to outgoing HTTP requests.
///
/// Implementations receive the backend-neutral [`HttpRequest`] and add headers
/// or query parameters. No HTTP backend type is part of this contract.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Returns the request with authentication credentials applied.
    async fn apply(&self, request: HttpRequest) -> EveryMapResult<HttpRequest>;
}
