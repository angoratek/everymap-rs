use async_trait::async_trait;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;

/// Extension trait for MapBox-specific geocoder capabilities.
///
/// These methods are not part of the core `Geocoder` trait because they
/// are MapBox-specific. Import this trait to access them:
///
/// ```ignore
/// use everymap_providers_mapbox::MapBoxGeocoderExt;
/// let results = geocoder.permanent_geocode(query, &options).await?;
/// ```
#[async_trait]
pub trait MapBoxGeocoderExt: Send + Sync {
    /// Use the MapBox permanent geocoding endpoint (batch lookups).
    async fn permanent_geocode(
        &self,
        query: &str,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::MapBoxSearchResponse>;

    /// Batch geocode up to 50 forward/reverse queries.
    async fn batch_geocode(
        &self,
        queries: &[String],
        language: Option<&str>,
    ) -> EveryMapResult<Vec<super::domain::search::MapBoxSearchResponse>>;
}

/// Extension trait for MapBox-specific routing capabilities.
#[async_trait]
pub trait MapBoxRouterExt: Send + Sync {
    /// Get a route with specific profile and options not in the core trait.
    async fn route_with_profile(
        &self,
        coordinates: &[Coordinate],
        profile: &str,
        alternatives: Option<u32>,
    ) -> EveryMapResult<super::domain::routing::MapBoxRouteResponse>;
}

// --- Extension trait implementations ---

#[async_trait]
impl MapBoxGeocoderExt for super::domain::search::MapBoxGeocoder {
    async fn permanent_geocode(
        &self,
        query: &str,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::MapBoxSearchResponse> {
        let url = format!("{}/search/geocode/v6/forward", self.base_url);
        // `permanent=true` is how Mapbox v6 records a query for permanent
        // storage; without it the same endpoint is the transient search.
        let mut params: Vec<(&str, String)> =
            vec![("q", query.to_string()), ("permanent", "true".to_string())];
        if let Some(lim) = limit {
            params.push(("limit", lim.to_string()));
        }
        if let Some(lang) = language {
            params.push(("language", lang.to_string()));
        }

        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);
        self.client.request_parsed_json(builder).await
    }

    async fn batch_geocode(
        &self,
        queries: &[String],
        language: Option<&str>,
    ) -> EveryMapResult<Vec<super::domain::search::MapBoxSearchResponse>> {
        /// The Geocoding v6 batch response: one FeatureCollection per query,
        /// in request order.
        #[derive(serde::Deserialize)]
        struct MapBoxBatchGeocodeResponse {
            #[serde(default)]
            batch: Vec<super::domain::search::MapBoxSearchResponse>,
        }

        let query_objects: Vec<serde_json::Value> = queries
            .iter()
            .map(|query| {
                let mut query_object = serde_json::json!({ "q": query });
                if let Some(language) = language {
                    query_object["language"] = serde_json::Value::from(language);
                }
                query_object
            })
            .collect();
        let url = format!("{}/search/geocode/v6/batch", self.base_url);
        let builder = self
            .client
            .build_request(everymap_core::http::Method::POST, &url)
            .json(&serde_json::json!({ "queries": query_objects }))?;
        let parsed: MapBoxBatchGeocodeResponse = self.client.request_parsed_json(builder).await?;
        Ok(parsed.batch)
    }
}

#[async_trait]
impl MapBoxRouterExt for super::domain::routing::MapBoxRouter {
    async fn route_with_profile(
        &self,
        coordinates: &[Coordinate],
        profile: &str,
        alternatives: Option<u32>,
    ) -> EveryMapResult<super::domain::routing::MapBoxRouteResponse> {
        if coordinates.len() < 2 {
            return Err(everymap_core::error::EveryMapError::provider(
                "mapbox",
                "INVALID_INPUT",
                "At least 2 coordinates required for routing",
            ));
        }

        let coords: String = coordinates
            .iter()
            .map(|c| format!("{},{}", c.lng, c.lat))
            .collect::<Vec<_>>()
            .join(";");
        let url = format!(
            "{}/directions/v5/mapbox/{}/{}",
            self.base_url, profile, coords
        );

        let mut params: Vec<(&str, String)> = vec![
            ("overview", "full".to_string()),
            ("geometries", "polyline".to_string()),
            ("steps", "true".to_string()),
        ];
        if let Some(alternative) = alternatives {
            params.push(("alternatives", alternative.to_string()));
        }

        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);
        self.client.request_parsed_json(builder).await
    }
}
