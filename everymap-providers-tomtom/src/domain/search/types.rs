use serde::{Deserialize, Serialize};

/// Response from TomTom Search API (geocode/reverse geocode).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSearchResponse {
    /// Search results (forward geocode).
    #[serde(default)]
    pub results: Vec<TomTomSearchResult>,
    /// Reverse geocode results (reverse geocode endpoint uses "addresses").
    #[serde(default)]
    pub addresses: Vec<TomTomReverseGeocodeResult>,
    /// Summary of the search.
    #[serde(default)]
    pub summary: Option<TomTomSearchSummary>,
}

/// Summary returned by TomTom Search API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSearchSummary {
    /// Number of results.
    #[serde(default, rename = "numResults")]
    pub num_results: Option<u32>,
    /// Query text.
    #[serde(default)]
    pub query: Option<String>,
    /// Query type.
    #[serde(default, rename = "queryType")]
    pub query_type: Option<String>,
    /// Total number of results available.
    #[serde(default, rename = "totalResults")]
    pub total_results: Option<u32>,
}

/// A single search result from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSearchResult {
    /// Result type (e.g., "Point Address", "Street", "Geography").
    #[serde(default, rename = "type")]
    pub result_type: Option<String>,
    /// Address information.
    #[serde(default)]
    pub address: Option<TomTomAddress>,
    /// Position (lat, lon).
    #[serde(default)]
    pub position: Option<TomTomPosition>,
    /// Viewport bounding box.
    #[serde(default, rename = "boundingBox")]
    pub bounding_box: Option<TomTomBoundingBox>,
    /// Distance in meters from the query point (for reverse geocode).
    #[serde(default)]
    pub dist: Option<f64>,
    /// Relevance score (0.0 to 1.0).
    #[serde(default)]
    pub score: Option<f64>,
    /// TomTom entity ID.
    #[serde(default)]
    pub id: Option<String>,
    /// Data sources (e.g., geometry).
    #[serde(default, rename = "dataSources")]
    pub data_sources: Option<serde_json::Value>,
}

/// Address details from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomAddress {
    /// Free-form address string.
    #[serde(default, rename = "freeformAddress")]
    pub freeform_address: Option<String>,
    /// Street name.
    #[serde(default, rename = "streetName")]
    pub street_name: Option<String>,
    /// Street number.
    #[serde(default, rename = "streetNumber")]
    pub street_number: Option<String>,
    /// Municipality (city).
    #[serde(default)]
    pub municipality: Option<String>,
    /// Country subdivision (state/province).
    #[serde(default, rename = "countrySubdivision")]
    pub country_subdivision: Option<String>,
    /// Country.
    #[serde(default)]
    pub country: Option<String>,
    /// ISO country code.
    #[serde(default, rename = "countryCode")]
    pub country_code: Option<String>,
    /// Postal code.
    #[serde(default, rename = "postalCode")]
    pub postal_code: Option<String>,
    /// Neighborhood.
    #[serde(default)]
    pub neighbourhood: Option<String>,
}

/// A single result from TomTom reverse geocode API.
/// The reverse geocode endpoint returns results under "addresses" with a different structure
/// than the forward geocode "results".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReverseGeocodeResult {
    /// Address information.
    #[serde(default)]
    pub address: Option<TomTomReverseGeocodeAddress>,
    /// Position as a "lat,lon" string (reverse geocode uses string format).
    #[serde(default)]
    pub position: Option<String>,
    /// TomTom entity ID.
    #[serde(default)]
    pub id: Option<String>,
    /// Distance in meters from the query point.
    #[serde(default)]
    pub dist: Option<f64>,
}

/// Address details from TomTom reverse geocode API.
/// This has a richer structure than the forward geocode address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReverseGeocodeAddress {
    /// Street name.
    #[serde(default, rename = "streetName")]
    pub street_name: Option<String>,
    /// Street number / name (alternative field).
    #[serde(default)]
    pub street: Option<String>,
    /// Route numbers.
    #[serde(default, rename = "routeNumbers")]
    pub route_numbers: Option<serde_json::Value>,
    /// ISO country code.
    #[serde(default, rename = "countryCode")]
    pub country_code: Option<String>,
    /// Country subdivision (state/province).
    #[serde(default, rename = "countrySubdivision")]
    pub country_subdivision: Option<String>,
    /// Country secondary subdivision.
    #[serde(default, rename = "countrySecondarySubdivision")]
    pub country_secondary_subdivision: Option<String>,
    /// Municipality (city).
    #[serde(default)]
    pub municipality: Option<String>,
    /// Postal code.
    #[serde(default, rename = "postalCode")]
    pub postal_code: Option<String>,
    /// Municipality subdivision.
    #[serde(default, rename = "municipalitySubdivision")]
    pub municipality_subdivision: Option<String>,
    /// Neighborhood.
    #[serde(default)]
    pub neighbourhood: Option<String>,
    /// Country name.
    #[serde(default)]
    pub country: Option<String>,
    /// ISO3 country code.
    #[serde(default, rename = "countryCodeISO3")]
    pub country_code_iso3: Option<String>,
    /// Free-form address string.
    #[serde(default, rename = "freeformAddress")]
    pub freeform_address: Option<String>,
    /// Bounding box.
    #[serde(default, rename = "boundingBox")]
    pub bounding_box: Option<TomTomReverseGeocodeBoundingBox>,
    /// Country subdivision name.
    #[serde(default, rename = "countrySubdivisionName")]
    pub country_subdivision_name: Option<String>,
    /// Country subdivision code.
    #[serde(default, rename = "countrySubdivisionCode")]
    pub country_subdivision_code: Option<String>,
    /// Local name.
    #[serde(default, rename = "localName")]
    pub local_name: Option<String>,
}

/// Bounding box from TomTom reverse geocode (uses northEast/southWest as "lat,lon" strings).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReverseGeocodeBoundingBox {
    /// North-east corner as "lat,lon" string.
    #[serde(default, rename = "northEast")]
    pub north_east: Option<String>,
    /// South-west corner as "lat,lon" string.
    #[serde(default, rename = "southWest")]
    pub south_west: Option<String>,
    /// Entity type.
    #[serde(default)]
    pub entity: Option<String>,
}

/// Position (lat/lon) from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomPosition {
    /// Latitude in degrees.
    #[serde(default)]
    pub lat: f64,
    /// Longitude in degrees.
    #[serde(default)]
    pub lon: f64,
}

/// Bounding box from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomBoundingBox {
    /// Top-left corner.
    #[serde(default, rename = "topLeftPoint")]
    pub top_left: TomTomPosition,
    /// Bottom-right corner.
    #[serde(default, rename = "btmRightPoint")]
    pub btm_right: TomTomPosition,
}