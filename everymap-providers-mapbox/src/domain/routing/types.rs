use serde::{Deserialize, Serialize};

/// Response from MapBox Directions API v5.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxRouteResponse {
    #[serde(default)]
    pub routes: Vec<MapBoxRoute>,
    /// "Ok" or error code.
    #[serde(default, rename = "code")]
    pub code: Option<String>,
    #[serde(default)]
    pub waypoints: Vec<MapBoxWaypoint>,
}

/// A single route from MapBox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxRoute {
    /// Route distance in meters.
    #[serde(default, rename = "distance")]
    pub distance: f64,
    /// Route duration in seconds.
    #[serde(default, rename = "duration")]
    pub duration: f64,
    /// Encoded polyline geometry.
    #[serde(default, rename = "geometry")]
    pub geometry: Option<String>,
    /// Route legs.
    #[serde(default)]
    pub legs: Vec<MapBoxRouteLeg>,
}

/// A leg of a MapBox route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxRouteLeg {
    #[serde(default, rename = "distance")]
    pub distance: f64,
    #[serde(default, rename = "duration")]
    pub duration: f64,
    #[serde(default, rename = "summary")]
    pub summary: Option<String>,
    #[serde(default)]
    pub steps: Vec<MapBoxRouteStep>,
}

/// A step/instruction in a MapBox route leg.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxRouteStep {
    #[serde(default, rename = "distance")]
    pub distance: f64,
    #[serde(default, rename = "duration")]
    pub duration: f64,
    #[serde(default, rename = "instruction")]
    pub instruction: Option<String>,
    #[serde(default, rename = "name")]
    pub name: Option<String>,
    /// Maneuver type.
    #[serde(default)]
    pub maneuver: Option<MapBoxManeuver>,
}

/// Maneuver information for a route step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxManeuver {
    #[serde(default, rename = "type")]
    pub maneuver_type: Option<String>,
    #[serde(default, rename = "modifier")]
    pub modifier: Option<String>,
    /// Location as [lng, lat].
    #[serde(default)]
    pub location: Option<Vec<f64>>,
}

/// A waypoint in the response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxWaypoint {
    #[serde(default)]
    pub name: Option<String>,
    /// Location as [lng, lat].
    #[serde(default)]
    pub location: Option<Vec<f64>>,
}
