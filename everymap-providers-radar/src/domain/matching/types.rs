use crate::domain::types::{RadarGeometry, RadarLocation, RadarMeta, RadarMetric};
use serde::{Deserialize, Serialize};

/// Response from Radar Route Match API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarRouteMatchResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub matched_path: Vec<RadarMatchedPoint>,
    pub road_attributes: Option<Vec<RadarRoadAttribute>>,
    pub geometry: Option<RadarGeometry>,
    pub distance: Option<RadarMetric>,
}

/// A point in the matched path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarMatchedPoint {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    pub original_index: Option<u32>,
}

/// Road attributes from route matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarRoadAttribute {
    pub road_class: Option<String>,
    pub speed_limit: Option<RadarMetric>,
    pub names: Option<Vec<String>>,
    pub start_location: Option<RadarLocation>,
    pub end_location: Option<RadarLocation>,
    pub original_index: Option<u32>,
}
