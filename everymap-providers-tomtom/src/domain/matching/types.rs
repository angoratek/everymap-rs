use serde::{Deserialize, Serialize};
use everymap_core::domains::matching::MatchedPoint;
use everymap_core::types::Coordinate;

impl From<TomTomSnapPoint> for MatchedPoint {
    fn from(sp: TomTomSnapPoint) -> Self {
        MatchedPoint {
            coordinate: sp.coordinate.map(|c| Coordinate::new(c.latitude, c.longitude)
                .unwrap_or(Coordinate::ORIGIN))
                .unwrap_or(Coordinate::ORIGIN),
            confidence: None,
            road_name: None,
        }
    }
}

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
    #[serde(default, rename = "routeOffset")]
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