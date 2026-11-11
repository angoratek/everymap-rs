use everymap_core::error::EveryMapResult;

everymap_core::provider_client! {
    /// The shared HTTP client for TomTom APIs.
    ///
    /// Delegates to `everymap_core::client::ProviderClient` for common
    /// request/response logic. Each domain module derives its endpoint from a
    /// configurable service root.
    TomTomClient, "tomtom"
}

impl TomTomClient {
    /// Sends a request and parses the JSON response body.
    ///
    /// Unlike [`Self::request_json`], a body that is not valid JSON becomes
    /// [`everymap_core::error::EveryMapError::SerializationError`], so a parse
    /// failure is distinguishable from a provider failure.
    ///
    /// # Parameters
    /// - `request` — the prepared request.
    ///
    /// # Returns
    /// The decoded body.
    ///
    /// # Errors
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`everymap_core::error::EveryMapError::SerializationError`] when the
    ///   body is not valid JSON.
    pub async fn request_parsed_json<T: serde::de::DeserializeOwned>(
        &self,
        request: everymap_core::http::HttpRequest,
    ) -> EveryMapResult<T> {
        let response = self.request(request).await?;
        Ok(serde_json::from_str(&response.text())?)
    }
}
