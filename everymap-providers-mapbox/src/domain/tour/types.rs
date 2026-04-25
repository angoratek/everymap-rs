use serde::{Deserialize, Serialize};

/// Response from MapBox Optimization API v1.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxOptimizationResponse {
    #[serde(default, rename = "code")]
    pub code: Option<String>,
    #[serde(default)]
    pub trips: Vec<MapBoxTrip>,
    #[serde(default)]
    pub waypoints: Vec<MapBoxOptWaypoint>,
}

/// An optimized trip from MapBox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxTrip {
    #[serde(default, rename = "distance")]
    pub distance: f64,
    #[serde(default, rename = "duration")]
    pub duration: f64,
    #[serde(default, rename = "geometry")]
    pub geometry: Option<String>,
    #[serde(default)]
    pub legs: Vec<serde_json::Value>,
}

/// A waypoint in the optimization response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxOptWaypoint {
    /// Waypoint index in the optimized order.
    #[serde(default, rename = "waypoint_index")]
    pub waypoint_index: Option<u64>,
    /// Original index in the input.
    #[serde(default, rename = "trips_index")]
    pub trips_index: Option<u64>,
    /// Location as [lng, lat].
    #[serde(default)]
    pub location: Option<Vec<f64>>,
    #[serde(default, rename = "name")]
    pub name: Option<String>,
}
