use serde::{Deserialize, Serialize};

/// Response from TomTom Waypoint Optimization API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomOptimizationResponse {
    #[serde(default)]
    pub optimized_waypoints: Vec<TomTomOptimizedWaypoint>,
    #[serde(default)]
    pub summary: Option<TomTomOptimizationSummary>,
}

/// An optimized waypoint from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomOptimizedWaypoint {
    #[serde(default, rename = "providedIndex")]
    pub provided_index: Option<u32>,
    #[serde(default)]
    pub point: Option<TomTomWaypointPosition>,
}

/// Waypoint position from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomWaypointPosition {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

/// Summary of the optimization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomOptimizationSummary {
    #[serde(default, rename = "lengthInMeters")]
    pub length_in_meters: Option<f64>,
    #[serde(default, rename = "travelTimeInSeconds")]
    pub travel_time_in_seconds: Option<f64>,
}