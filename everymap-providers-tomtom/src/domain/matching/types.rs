use serde::{Deserialize, Serialize};

/// Response from TomTom Snap to Roads API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSnapResponse {
    #[serde(default, rename = "snappedPoints")]
    pub snapped_points: Vec<TomTomSnapPoint>,
}

/// A snapped point from TomTom Snap to Roads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSnapPoint {
    #[serde(default)]
    pub coordinate: Option<TomTomSnapCoordinate>,
    #[serde(default, rename = "originalIndex")]
    pub original_index: Option<u32>,
    #[serde(default)]
    pub route_offset: Option<f64>,
}

/// Coordinate from TomTom Snap to Roads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSnapCoordinate {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}