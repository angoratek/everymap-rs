use serde::{Deserialize, Serialize};

/// Response from TomTom Waypoint Optimization API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomOptimizationResponse {
    /// Optimized order of waypoint indices (e.g., [0, 2, 1, 3]).
    #[serde(default, rename = "optimizedOrder")]
    pub optimized_order: Vec<u32>,
    /// Excluded waypoints (if importance-based exclusion was used).
    #[serde(default, rename = "excludedWaypoints")]
    pub excluded_waypoints: Vec<serde_json::Value>,
    /// Summary with travel times and route lengths (requires extensions).
    #[serde(default)]
    pub summary: Option<TomTomOptimizationSummary>,
}

/// Summary of the optimization (returned when extensions are requested).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomOptimizationSummary {
    /// Route-level summary.
    #[serde(default, rename = "routeSummary")]
    pub route_summary: Option<TomTomTourRouteSummary>,
}

/// Route-level summary for tour optimization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomTourRouteSummary {
    /// Length in meters.
    #[serde(default, rename = "lengthInMeters")]
    pub length_in_meters: Option<f64>,
    /// Travel time in seconds.
    #[serde(default, rename = "travelTimeInSeconds")]
    pub travel_time_in_seconds: Option<f64>,
}