//! An authentication provider for services that need no credential.

use crate::auth::AuthProvider;
use crate::error::EveryMapResult;
use crate::http::HttpRequest;
use async_trait::async_trait;

/// Authentication for a keyless or self-hosted service.
///
/// A provider that has no credential seam (a self-hosted OSRM or Valhalla
/// deployment, a URL-template tile source) uses this so it shares the same
/// client pipeline as every credentialed provider without inventing a
/// placeholder key.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoAuthProvider;

impl NoAuthProvider {
    /// Creates a keyless authentication provider.
    ///
    /// # Returns
    /// A provider that leaves every request unchanged.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AuthProvider for NoAuthProvider {
    async fn apply(&self, request: HttpRequest) -> EveryMapResult<HttpRequest> {
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_auth_leaves_the_request_unchanged() {
        let request = HttpRequest::get("https://example.com/route");
        let applied = NoAuthProvider::new().apply(request.clone()).await.unwrap();
        assert_eq!(applied.url(), request.url());
        assert!(applied.headers().iter().next().is_none());
    }
}
