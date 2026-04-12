use serde::{Deserialize, Serialize};

/// Options for Google Roads API speedLimits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleAttributeOptions {
    /// Place IDs for which to retrieve speed limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_ids: Option<Vec<String>>,
    /// Path of latitude/longitude pairs for snapped speed limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Units for speed limits: "KPH" or "MPH". Defaults to "KPH".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

/// Response from Google Roads API speedLimits endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSpeedLimitsResponse {
    /// Speed limit results.
    #[serde(default, rename = "speedLimits")]
    pub speed_limits: Vec<GoogleSpeedLimit>,
    /// Snapped points (when path is provided).
    #[serde(default, rename = "snappedPoints")]
    pub snapped_points: Vec<GoogleSnappedSpeedPoint>,
}

/// A speed limit result from the Roads API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSpeedLimit {
    /// Place ID of the road segment.
    #[serde(default, rename = "placeId")]
    pub place_id: Option<String>,
    /// Speed limit value.
    #[serde(default, rename = "speedLimit")]
    pub speed_limit: Option<f64>,
    /// Units of the speed limit ("KPH" or "MPH").
    #[serde(default)]
    pub units: Option<String>,
}

/// A snapped point with speed limit data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSnappedSpeedPoint {
    /// Location of the snapped point.
    #[serde(default)]
    pub location: GoogleSpeedLocation,
    /// Original index in the path.
    #[serde(default, rename = "originalIndex")]
    pub original_index: Option<u32>,
    /// Place ID of the road segment.
    #[serde(default, rename = "placeId")]
    pub place_id: Option<String>,
}

/// A lat/lng location for speed limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSpeedLocation {
    /// Latitude in degrees.
    #[serde(default)]
    pub latitude: f64,
    /// Longitude in degrees.
    #[serde(default)]
    pub longitude: f64,
}