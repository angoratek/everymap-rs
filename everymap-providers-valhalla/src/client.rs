//! Configuration and the keyless HTTP client for a Valhalla server.
//!
//! Valhalla is self-hosted and authenticates at the network layer when the
//! operator requires it, so the client has no credential seam. It shares the
//! workspace request pipeline through [`everymap_core::client::ProviderClient`]
//! with [`NoAuthProvider`], which gives the provider the same retries,
//! timeouts, verbose logging and request observation as every credentialed
//! provider.

use everymap_core::auth::NoAuthProvider;
use everymap_core::client::ProviderClient;
use everymap_core::http::HttpTransport;
use std::sync::Arc;

/// Base URL of a locally running Valhalla server used when no configuration is
/// supplied.
///
/// The reference Valhalla container publishes its HTTP service on port 8002.
pub const DEFAULT_VALHALLA_BASE_URL: &str = "http://localhost:8002";

/// `User-Agent` value sent when the configuration carries none.
pub const DEFAULT_VALHALLA_USER_AGENT: &str =
    concat!("everymap-valhalla-provider/", env!("CARGO_PKG_VERSION"));

/// Name this provider reports in errors and request observations.
pub const VALHALLA_PROVIDER_NAME: &str = "valhalla";

everymap_core::provider_client! {
    /// The shared HTTP client for a Valhalla server.
    ///
    /// The type wraps [`ProviderClient`], so the request pipeline is identical
    /// to every other provider's. Valhalla is keyless: construct it with
    /// [`NoAuthProvider`], or use [`ValhallaClient::keyless`].
    ValhallaClient, "valhalla"
}

impl ValhallaClient {
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
                VALHALLA_PROVIDER_NAME,
            ),
        }
    }
}

/// Connection settings for [`crate::ValhallaRouter`].
///
/// Valhalla is keyless; the operator supplies the address of their own server
/// and the application name the server logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValhallaProviderConfiguration {
    /// Server root such as `http://localhost:8002`. The `/route` path is
    /// appended to it.
    pub base_url: String,
    /// Value every request sends in the `User-Agent` header.
    pub user_agent: String,
}

impl Default for ValhallaProviderConfiguration {
    /// Configuration for the reference Valhalla container on the local host.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_VALHALLA_BASE_URL.to_string(),
            user_agent: DEFAULT_VALHALLA_USER_AGENT.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ValhallaProviderConfiguration, DEFAULT_VALHALLA_BASE_URL};

    #[test]
    fn default_configuration_targets_the_local_valhalla_port() {
        let configuration = ValhallaProviderConfiguration::default();
        assert_eq!(configuration.base_url, DEFAULT_VALHALLA_BASE_URL);
        assert_eq!(configuration.base_url, "http://localhost:8002");
        assert!(!configuration.user_agent.is_empty());
    }
}
