use serde::{Deserialize, Serialize};
use everymap_core::domains::matching::MatchedPoint;
use everymap_core::types::Coordinate;

impl From<MapBoxTracepoint> for MatchedPoint {
    fn from(tp: MapBoxTracepoint) -> Self {
        let coordinate = tp.location.as_ref()
            .and_then(|loc| {
                if loc.len() >= 2 {
                    Some(Coordinate::new(loc[1], loc[0])
                        .unwrap_or(Coordinate::ORIGIN))
                } else {
                    None
                }
            })
            .unwrap_or(Coordinate::ORIGIN);

        MatchedPoint {
            coordinate,
            confidence: tp.distance.map(|d| 1.0 - (d / 100.0).min(1.0)),
            road_name: tp.name,
        }
    }
}

/// Response from MapBox Map Matching API v5.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxMatchResponse {
    #[serde(default, rename = "code")]
    pub code: Option<String>,
    #[serde(default)]
    pub matchings: Vec<MapBoxMatching>,
    #[serde(default)]
    pub tracepoints: Vec<MapBoxTracepoint>,
}

/// A matched route from MapBox Map Matching.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxMatching {
    #[serde(default, rename = "confidence")]
    pub confidence: Option<f64>,
    #[serde(default, rename = "distance")]
    pub distance: f64,
    #[serde(default, rename = "duration")]
    pub duration: f64,
    #[serde(default, rename = "geometry")]
    pub geometry: Option<String>,
    #[serde(default)]
    pub legs: Vec<MapBoxMatchLeg>,
}

/// A leg in a matched route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxMatchLeg {
    #[serde(default, rename = "distance")]
    pub distance: f64,
    #[serde(default, rename = "duration")]
    pub duration: f64,
    #[serde(default)]
    pub steps: Vec<serde_json::Value>,
}

/// A tracepoint from MapBox Map Matching.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxTracepoint {
    /// Waypoint index in the original trace.
    #[serde(default, rename = "waypoint_index")]
    pub waypoint_index: Option<u64>,
    /// Matched location as [lng, lat].
    #[serde(default)]
    pub location: Option<Vec<f64>>,
    /// Name of the matched road.
    #[serde(default, rename = "name")]
    pub name: Option<String>,
    /// Distance from the original point to the matched point (meters).
    #[serde(default, rename = "distance")]
    pub distance: Option<f64>,
}