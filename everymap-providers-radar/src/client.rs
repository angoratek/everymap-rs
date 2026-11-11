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

    /// Sends a POST request with a JSON body and parses the response.
    ///
    /// # Parameters
    /// - `url` — the full endpoint URL.
    /// - `body` — the request body serialized as JSON.
    ///
    /// # Returns
    /// The decoded response body.
    ///
    /// # Errors
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`everymap_core::error::EveryMapError::SerializationError`] when the
    ///   body cannot be serialized or the response body is not valid JSON.
    pub async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> EveryMapResult<T> {
        let request = self
            .build_request(everymap_core::http::Method::Post, url)
            .json(body)?;
        self.request_parsed_json(request).await
    }
}
