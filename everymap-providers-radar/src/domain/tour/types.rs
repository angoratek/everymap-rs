use serde::{Deserialize, Serialize};
use crate::domain::types::{RadarMeta, RadarMetric, RadarLocation, RadarGeometry};

/// Response from Radar Optimize Route API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOptimizeResponse {
    pub meta: RadarMeta,
    pub route: RadarOptimizedRoute,
}

/// An optimized route from Radar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOptimizedRoute {
    pub distance: RadarMetric,
    pub duration: RadarMetric,
    #[serde(default)]
    pub legs: Vec<RadarOptimizedLeg>,
}

/// A leg of an optimized route.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOptimizedLeg {
    pub start_location: RadarLocation,
    pub end_location: RadarLocation,
    #[serde(default)]
    pub start_index: u32,
    #[serde(default)]
    pub end_index: u32,
    pub distance: RadarMetric,
    pub duration: RadarMetric,
    pub geometry: Option<RadarGeometry>,
}