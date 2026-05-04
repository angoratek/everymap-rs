use async_trait::async_trait;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;

/// Extension trait for TomTom-specific geocoder capabilities.
///
/// These methods are not part of the core `Geocoder` trait because they
/// are TomTom-specific. Import this trait to access them:
///
/// ```ignore
/// use everymap_providers_tomtom::TomTomGeocoderExt;
/// let results = geocoder.nearby_search(coordinate, 5000, &options).await?;
/// ```
#[async_trait]
pub trait TomTomGeocoderExt: Send + Sync {
    /// Search for POIs near a coordinate within a radius (meters).
    async fn nearby_search(
        &self,
        location: &Coordinate,
        radius: u32,
        query: &str,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::TomTomSearchResponse>;

    /// Search for POIs by category (e.g., "RESTAURANT", "PARKING_GARAGE").
    async fn category_search(
        &self,
        category: &str,
        location: &Coordinate,
        radius: Option<u32>,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::TomTomSearchResponse>;
}

/// Extension trait for TomTom-specific traffic capabilities.
///
/// Provides direct access to raw TomTom flow and incident data types.
#[async_trait]
pub trait TomTomTrafficExt: Send + Sync {
    /// Get raw traffic flow data from TomTom.
    async fn get_flow(
        &self,
        location: &Coordinate,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::traffic::TomTomFlowResponse>;

    /// Get raw traffic incidents from TomTom.
    async fn get_incidents(
        &self,
        bbox: &str,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::traffic::TomTomIncidentsResponse>;
}

// --- Extension trait implementations ---

#[async_trait]
impl TomTomGeocoderExt for super::domain::search::TomTomGeocoder {
    async fn nearby_search(
        &self,
        location: &Coordinate,
        radius: u32,
        query: &str,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::TomTomSearchResponse> {
        let url = format!("{}/search/2/nearbySearch/.json", self.base_url);
        let mut params: Vec<(&str, String)> = vec![
            ("lat", location.lat.to_string()),
            ("lon", location.lng.to_string()),
            ("radius", radius.to_string()),
            ("query", query.to_string()),
        ];
        if let Some(lim) = limit {
            params.push(("limit", lim.to_string()));
        }
        if let Some(lang) = language {
            params.push(("language", lang.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    async fn category_search(
        &self,
        category: &str,
        location: &Coordinate,
        radius: Option<u32>,
        limit: Option<u32>,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::search::TomTomSearchResponse> {
        let url = format!(
            "{}/search/2/categorySearch/{}.json",
            self.base_url, category
        );
        let mut params: Vec<(&str, String)> = vec![
            ("lat", location.lat.to_string()),
            ("lon", location.lng.to_string()),
        ];
        if let Some(r) = radius {
            params.push(("radius", r.to_string()));
        }
        if let Some(lim) = limit {
            params.push(("limit", lim.to_string()));
        }
        if let Some(lang) = language {
            params.push(("language", lang.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }
}

#[async_trait]
impl TomTomTrafficExt for super::domain::traffic::TomTomTraffic {
    async fn get_flow(
        &self,
        location: &Coordinate,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::traffic::TomTomFlowResponse> {
        let url = format!(
            "{}/traffic/services/4/flowSegmentData/absolute/10/json",
            self.base_url
        );
        let mut params: Vec<(&str, String)> =
            vec![("point", format!("{},{}", location.lat, location.lng))];
        if let Some(lang) = language {
            params.push(("language", lang.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    async fn get_incidents(
        &self,
        bbox: &str,
        language: Option<&str>,
    ) -> EveryMapResult<super::domain::traffic::TomTomIncidentsResponse> {
        let url = format!("{}/traffic/services/5/incidentDetails", self.base_url);
        let mut params: Vec<(&str, String)> = vec![
            ("bbox", bbox.to_string()),
            ("fields", "{incidents{type,geometry{id,coordinates},severity,description,from,to,startTime,endTime}}".to_string()),
        ];
        if let Some(lang) = language {
            params.push(("language", lang.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }
}
