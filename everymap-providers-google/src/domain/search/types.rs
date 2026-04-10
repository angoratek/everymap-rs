use serde::{Deserialize, Serialize};
use crate::domain::geo::{GoogleLatLng, GoogleBounds};

// ============================================================================
// Response types — Google Geocoding API
// ============================================================================

/// Full response from the Google Geocoding API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeocodeResponse {
    #[serde(default)]
    pub results: Vec<GoogleGeocodeResult>,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "error_message")]
    pub error_message: Option<String>,
}

/// A single geocoding result from Google.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeocodeResult {
    #[serde(default)]
    pub formatted_address: Option<String>,
    #[serde(default)]
    pub geometry: Option<GoogleGeometry>,
    #[serde(default)]
    pub place_id: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub address_components: Vec<GoogleAddressComponent>,
    #[serde(default)]
    pub partial_match: Option<bool>,
    #[serde(default)]
    pub plus_code: Option<GooglePlusCode>,
}

/// Geometry container with location and viewport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeometry {
    #[serde(default)]
    pub location: Option<GoogleLatLng>,
    #[serde(default)]
    pub location_type: Option<String>,
    #[serde(default)]
    pub viewport: Option<GoogleBounds>,
    #[serde(default)]
    pub bounds: Option<GoogleBounds>,
}

/// An address component with type information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAddressComponent {
    #[serde(default)]
    pub long_name: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
}

/// Plus code (Open Location Code).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooglePlusCode {
    #[serde(default)]
    pub compound_code: Option<String>,
    #[serde(default)]
    pub global_code: Option<String>,
}