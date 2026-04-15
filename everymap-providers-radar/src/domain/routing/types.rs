use serde::{Deserialize, Serialize};
use crate::domain::types::{RadarMeta, RadarMetric, RadarLocation, RadarGeometry};

/// Response from Radar Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDirectionsResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub routes: Vec<RadarDirectionsRoute>,
}

/// A route from Radar Directions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDirectionsRoute {
    pub duration: RadarMetric,
    pub distance: RadarMetric,
    #[serde(default)]
    pub legs: Vec<RadarDirectionsLeg>,
    pub geometry: Option<RadarGeometry>,
}

/// A leg of a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDirectionsLeg {
    pub start_location: RadarLocation,
    pub end_location: RadarLocation,
    pub duration: RadarMetric,
    pub distance: RadarMetric,
    pub geometry: Option<RadarGeometry>,
    #[serde(default)]
    pub steps: Vec<RadarDirectionsStep>,
}

/// A step within a route leg (turn-by-turn).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDirectionsStep {
    pub distance: RadarMetric,
    pub duration: RadarMetric,
    pub start_location: RadarLocation,
    pub end_location: RadarLocation,
    #[serde(default)]
    pub bearing_before: f64,
    #[serde(default)]
    pub bearing_after: f64,
    pub instructions: Option<String>,
    pub banner_instructions: Option<String>,
    pub voice_instructions: Option<String>,
    pub geometry: Option<RadarGeometry>,
    pub mode: Option<String>,
    pub maneuver: Option<String>,
    pub street_name: Option<String>,
    pub exit_name: Option<String>,
}

/// Response from Radar Distance API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDistanceResponse {
    pub meta: RadarMeta,
    pub routes: RadarDistanceRoutes,
}

/// Distance routes (by mode).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarDistanceRoutes {
    pub geodesic: Option<RadarModeDistance>,
    pub car: Option<RadarModeDistance>,
    pub truck: Option<RadarModeDistance>,
    pub foot: Option<RadarModeDistance>,
    pub bike: Option<RadarModeDistance>,
}

/// Distance/duration for a specific mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarModeDistance {
    pub distance: RadarMetric,
    pub duration: Option<RadarMetric>,
}

/// Response from Radar Matrix API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarMatrixResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub origins: Vec<RadarLocation>,
    #[serde(default)]
    pub destinations: Vec<RadarLocation>,
    #[serde(default)]
    pub matrix: Vec<Vec<RadarMatrixEntry>>,
}

/// A single entry in the distance/duration matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarMatrixEntry {
    pub distance: RadarMetric,
    pub duration: RadarMetric,
    #[serde(default)]
    pub origin_index: u32,
    #[serde(default)]
    pub destination_index: u32,
}