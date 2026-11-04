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

            #[cfg(not(target_arch = "wasm32"))]
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
                method: $crate::http::Method,
                url: &str,
            ) -> $crate::http::HttpRequest {
                self.inner.build_request(method, url)
            }

            /// Sends a request, applying authentication first.
            pub async fn request(
                &self,
                request: $crate::http::HttpRequest,
            ) -> $crate::error::EveryMapResult<$crate::http::HttpResponse> {
                self.inner.request(request).await
            }

            /// Sends a request and deserializes the JSON response into `T`.
            pub async fn request_json<T: ::serde::de::DeserializeOwned>(
                &self,
                request: $crate::http::HttpRequest,
            ) -> $crate::error::EveryMapResult<T> {
                self.inner.request_json(request).await
            }

            /// Installs a token bucket that paces every request this client
            /// sends.
            pub fn set_rate_limiter(
                &mut self,
                rate_limiter: ::std::sync::Arc<$crate::client::TokenBucket>,
            ) {
                self.inner.set_rate_limiter(rate_limiter);
            }

            /// Replaces the retry policy applied to idempotent requests.
            pub fn set_retry_policy(&mut self, retry_policy: $crate::client::RetryPolicy) {
                self.inner.set_retry_policy(retry_policy);
            }

            /// Sets the per-request timeout, or removes it.
            pub fn set_request_timeout(&mut self, timeout: ::std::option::Option<::std::time::Duration>) {
                self.inner.set_request_timeout(timeout);
            }

            /// Installs an observer that receives every attempt's redacted
            /// facts, replacing any earlier observer on this client.
            pub fn set_observer(
                &mut self,
                observer: ::std::sync::Arc<dyn $crate::client::RequestObserver>,
            ) {
                self.inner.set_observer(observer);
            }
        }
    };
}
