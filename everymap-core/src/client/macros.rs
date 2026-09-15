//! Declarative macro for generating provider client wrapper boilerplate.

/// Generates a provider client struct that wraps `everymap_core::client::ProviderClient`.
///
/// Expands to a public struct holding a private `inner: ProviderClient`, a private
/// `PROVIDER_NAME` const, and the standard methods shared by every provider client
/// wrapper: `new`, `with_client_builder`, `set_verbose`, `is_verbose`,
/// `build_request`, `request`, and `request_json`.
///
/// Provider-specific methods (e.g. Radar's `post_json`) stay outside the macro as
/// inherent methods on the generated struct in the same module, accessing `inner`
/// directly.
///
/// Usage:
/// ```ignore
/// everymap_core::provider_client! {
///     /// The shared HTTP client for Google Maps APIs.
///     GoogleClient, "google"
/// }
/// ```
#[macro_export]
macro_rules! provider_client {
    (
        $(#[$struct_documentation:meta])*
        $struct_name:ident, $provider_name:literal
    ) => {
        $(#[$struct_documentation])*
        pub struct $struct_name {
            inner: $crate::client::ProviderClient,
        }

        const PROVIDER_NAME: &str = $provider_name;

        impl $struct_name {
            #[doc = concat!("Creates a new `", stringify!($struct_name), "` with the given authentication provider.")]
            pub fn new(
                auth_provider: ::std::sync::Arc<dyn $crate::auth::AuthProvider>,
            ) -> Self {
                Self {
                    inner: $crate::client::ProviderClient::new(
                        auth_provider,
                        PROVIDER_NAME,
                    ),
                }
            }

            #[doc = concat!("Creates a `", stringify!($struct_name), "` with a custom `reqwest::Client` configuration.")]
            pub fn with_client_builder(
                builder: ::reqwest::ClientBuilder,
                auth_provider: ::std::sync::Arc<dyn $crate::auth::AuthProvider>,
            ) -> $crate::error::EveryMapResult<Self> {
                Ok(Self {
                    inner: $crate::client::ProviderClient::with_client_builder(
                        builder,
                        auth_provider,
                        PROVIDER_NAME,
                    )?,
                })
            }

            /// Enable or disable verbose output (request/response logging to stderr).
            pub fn set_verbose(&mut self, verbose: bool) {
                self.inner.set_verbose(verbose);
            }

            /// Whether verbose mode is enabled.
            pub fn is_verbose(&self) -> bool {
                self.inner.is_verbose()
            }

            /// Builds a request to the given full URL with the specified HTTP method.
            pub fn build_request(
                &self,
                method: ::reqwest::Method,
                url: &str,
            ) -> ::reqwest::RequestBuilder {
                self.inner.build_request(method, url)
            }

            /// Sends a request, applying authentication first.
            pub async fn request(
                &self,
                builder: ::reqwest::RequestBuilder,
            ) -> $crate::error::EveryMapResult<::reqwest::Response> {
                self.inner.request(builder).await
            }

            /// Sends a request and deserializes the JSON response into `T`.
            pub async fn request_json<T: ::serde::de::DeserializeOwned>(
                &self,
                builder: ::reqwest::RequestBuilder,
            ) -> $crate::error::EveryMapResult<T> {
                self.inner.request_json(builder).await
            }
        }
    };
}
