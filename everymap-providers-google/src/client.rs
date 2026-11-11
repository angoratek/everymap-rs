use everymap_core::error::EveryMapResult;

everymap_core::provider_client! {
    /// The shared HTTP client for Google Maps APIs.
    ///
    /// Delegates to `everymap_core::client::ProviderClient` for common
    /// request/response logic. Each domain module derives its endpoint from a
    /// configurable service root.
    GoogleClient, "google"
}

/// Builds an endpoint URL from a service root and a documented path.
///
/// Every Google domain follows one convention: the configurable base value is
/// a service root such as `https://maps.googleapis.com`, and the endpoint is
/// the root followed by the API path. A root with a trailing slash is
/// accepted.
///
/// # Parameters
/// - `service_root` — the scheme and host serving the API.
/// - `path` — the documented API path, starting with a slash.
///
/// # Returns
/// The joined endpoint URL.
pub(crate) fn endpoint_url(service_root: &str, path: &str) -> String {
    format!("{}{}", service_root.trim_end_matches('/'), path)
}

impl GoogleClient {
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

#[cfg(test)]
mod tests {
    use super::endpoint_url;

    #[test]
    fn the_endpoint_is_the_root_plus_the_documented_path() {
        assert_eq!(
            endpoint_url("https://maps.googleapis.com", "/maps/api/geocode/json"),
            "https://maps.googleapis.com/maps/api/geocode/json"
        );
    }

    #[test]
    fn a_trailing_slash_on_the_root_is_accepted() {
        assert_eq!(
            endpoint_url("http://127.0.0.1:8080/", "/maps/api/geocode/json"),
            "http://127.0.0.1:8080/maps/api/geocode/json"
        );
    }
}
