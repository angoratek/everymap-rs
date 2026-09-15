everymap_core::provider_client! {
    /// The shared HTTP client for TomTom APIs.
    ///
    /// Delegates to `everymap_core::client::ProviderClient` for common
    /// request/response logic. Each domain module constructs its own
    /// base URL per the TomTom API specification.
    TomTomClient, "tomtom"
}
