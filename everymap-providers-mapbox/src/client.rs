everymap_core::provider_client! {
    /// The shared HTTP client for MapBox APIs.
    ///
    /// Delegates to `everymap_core::client::ProviderClient` for common
    /// request/response logic. Each domain module constructs its own
    /// base URL per the MapBox API specification.
    MapBoxClient, "mapbox"
}
