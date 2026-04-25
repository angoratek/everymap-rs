use serde::{Deserialize, Serialize};

/// Options for Google Roads API snapToRoads.
#[derive(Debug, Clone, Default)]
pub struct GoogleMatchOptions {
    /// Whether to interpolate the path between snapped points.
    pub interpolate: bool,
    /// Snapping type (not commonly used, reserved).
    pub snapping: Option<String>,
}

/// Response from Google Roads API snapToRoads endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSnapResponse {
    #[serde(default, rename = "snappedPoints")]
    pub snapped_points: Vec<GoogleSnappedPoint>,
}

/// A snapped point from the Roads API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleSnappedPoint {
    #[serde(default)]
    pub location: GoogleLocation,
    #[serde(default, rename = "originalIndex")]
    pub original_index: Option<u32>,
    #[serde(default, rename = "placeId")]
    pub place_id: Option<String>,
}

/// A lat/lng location from Google APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleLocation {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}
