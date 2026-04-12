use serde::{Deserialize, Serialize};

/// Response from TomTom Reachable Range API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReachableRangeResponse {
    #[serde(default, rename = "reachableRange")]
    pub reachable_range: Option<TomTomReachableRange>,
    #[serde(default)]
    pub report: Option<TomTomReachableReport>,
}

/// Reachable range data from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReachableRange {
    #[serde(default)]
    pub center: Option<TomTomReachableCenter>,
    #[serde(default)]
    pub boundary: Vec<TomTomReachableBoundary>,
}

/// Center point of reachable range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReachableCenter {
    #[serde(default, rename = "latitude")]
    pub lat: f64,
    #[serde(default, rename = "longitude")]
    pub lng: f64,
}

/// A boundary point of the reachable range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReachableBoundary {
    #[serde(default, rename = "latitude")]
    pub lat: f64,
    #[serde(default, rename = "longitude")]
    pub lng: f64,
}

/// Report data for reachable range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomReachableReport {
    #[serde(default, rename = "effectiveSettings")]
    pub effective_settings: Option<serde_json::Value>,
}