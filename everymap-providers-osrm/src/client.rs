//! Configuration and the keyless HTTP client for an OSRM deployment.
//!
//! OSRM is self-hosted and authenticates at the network layer when the
//! operator requires it, so the client has no credential seam. It shares the
//! workspace request pipeline through [`everymap_core::client::ProviderClient`]
//! with [`NoAuthProvider`], which gives the provider the same retries,
//! timeouts, verbose logging and request observation as every credentialed
//! provider.

use everymap_core::auth::NoAuthProvider;
use everymap_core::client::ProviderClient;
use everymap_core::http::HttpTransport;
use std::sync::Arc;

/// Base URL of a locally hosted OSRM server used when no configuration is
/// supplied.
///
/// A local OSRM installation serves its HTTP API on port 5000.
pub const DEFAULT_OSRM_BASE_URL: &str = "http://localhost:5000";

/// `User-Agent` value sent when the configuration carries none.
pub const DEFAULT_OSRM_USER_AGENT: &str =
    concat!("everymap-osrm-provider/", env!("CARGO_PKG_VERSION"));

/// Name this provider reports in errors and request observations.
pub const OSRM_PROVIDER_NAME: &str = "osrm";

everymap_core::provider_client! {
    /// The shared HTTP client for an OSRM deployment.
    ///
    /// The type wraps [`ProviderClient`], so the request pipeline is identical
    /// to every other provider's. OSRM is keyless: construct it with
    /// [`NoAuthProvider`], or use [`OsrmClient::keyless`].
    OsrmClient, "osrm"
}

impl OsrmClient {
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
                OSRM_PROVIDER_NAME,
            ),
        }
    }
}

/// Connection settings for one OSRM deployment.
///
/// OSRM is keyless, so the deployment URL is the only address information the
/// service needs. `base_url` may name any host serving the OSRM HTTP API
/// (`http://host:5000`, `https://routing.example.com`, and so on) and must be
/// an absolute URL without the route service path.
///
/// `user_agent` identifies the client to the deployment's operator; an empty
/// value suppresses the header entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsrmProviderConfiguration {
    /// Root URL of the OSRM deployment, without the route service path.
    pub base_url: String,
    /// `User-Agent` header value sent with every request.
    pub user_agent: String,
}

impl Default for OsrmProviderConfiguration {
    /// Settings for a local OSRM installation on the default port.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_OSRM_BASE_URL.to_string(),
            user_agent: DEFAULT_OSRM_USER_AGENT.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OsrmProviderConfiguration, DEFAULT_OSRM_BASE_URL};

    #[test]
    fn default_configuration_targets_the_local_osrm_port() {
        let configuration = OsrmProviderConfiguration::default();
        assert_eq!(configuration.base_url, DEFAULT_OSRM_BASE_URL);
        assert_eq!(configuration.base_url, "http://localhost:5000");
        assert!(!configuration.user_agent.is_empty());
    }
}
