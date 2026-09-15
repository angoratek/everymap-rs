use everymap_core::error::EveryMapResult;

everymap_core::provider_client! {
    /// The shared HTTP client for Radar APIs.
    ///
    /// Delegates to `everymap_core::client::ProviderClient` for common
    /// request/response logic. Handles authentication via the `Authorization`
    /// header (configured via `HeaderAuthProvider`). Each domain module
    /// constructs its own URL path per the Radar API specification.
    RadarClient, "radar"
}

impl RadarClient {
    /// Sends a POST request with a JSON body and deserializes the response.
    pub async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> EveryMapResult<T> {
        self.inner.post_json(url, body).await
    }
}
