//! Keyless HTTP client for URL-template vector tile sources.
//!
//! A URL-template tileset may authenticate through a query parameter or a
//! header baked into the template, or may be entirely public; the client has
//! no credential seam of its own. It shares the workspace request pipeline
//! through [`everymap_core::client::ProviderClient`] with
//! [`everymap_core::auth::NoAuthProvider`], which gives the provider the same
//! retries, timeouts, pacing, verbose logging and observation as every
//! credentialed provider.

use everymap_core::auth::NoAuthProvider;
use everymap_core::client::ProviderClient;
use everymap_core::http::HttpTransport;
use std::sync::Arc;

/// Name this provider reports in errors and request observations.
pub const URL_TEMPLATE_VECTOR_PROVIDER_NAME: &str = "url-template-vector";

everymap_core::provider_client! {
    /// The shared HTTP client for a URL-template vector tile source.
    ///
    /// The type wraps [`ProviderClient`], so the request pipeline is identical
    /// to every other provider's. The source is keyless: construct it with
    /// [`NoAuthProvider`], or use [`UrlTemplateVectorClient::keyless`].
    UrlTemplateVectorClient, "url-template-vector"
}

impl UrlTemplateVectorClient {
    /// Creates a keyless client on the target's default transport.
    ///
    /// # Returns
    /// A client that sends requests with the platform transport and no
    /// credential.
    pub fn keyless() -> Self {
        Self::new(Arc::new(NoAuthProvider::new()))
    }

    /// Creates a keyless client that executes requests through an explicit
    /// transport.
    ///
    /// # Parameters
    /// - `transport` — the transport that sends each request. Hosts inject
    ///   their platform transport and tests inject a fake.
    ///
    /// # Returns
    /// A client with no credential and the shared pipeline defaults.
    pub fn with_transport(transport: Arc<dyn HttpTransport>) -> Self {
        Self {
            inner: ProviderClient::with_transport(
                transport,
                Arc::new(NoAuthProvider::new()),
                URL_TEMPLATE_VECTOR_PROVIDER_NAME,
            ),
        }
    }
}
