use serde::{Deserialize, Serialize};
use everymap_core::domains::matching::MatchedPoint;
use everymap_core::types::Coordinate;

/// Response from TomTom Snap to Roads API (Synchronous).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomSnapResponse {
    /// Projected (snapped) points as GeoJSON Features.
    #[serde(default, rename = "projectedPoints")]
    pub projected_points: Vec<TomTomProjectedPoint>,
    /// Route segments as GeoJSON Features.
    #[serde(default)]
    pub route: Vec<serde_json::Value>,
    /// Distance summary.
    #[serde(default)]
    pub distances: Option<TomTomDistances>,
}

/// A projected point from TomTom Snap to Roads (GeoJSON Feature).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomProjectedPoint {
    /// GeoJSON geometry.
    #[serde(default)]
    pub geometry: Option<TomTomProjectedGeometry>,
    /// Properties including snap result and route index.
    #[serde(default)]
    pub properties: Option<TomTomProjectedProperties>,
}

/// Geometry for a projected point (GeoJSON Point).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomProjectedGeometry {
    /// Point type.
    #[serde(default, rename = "type")]
    pub geo_type: Option<String>,
    /// Coordinates as [longitude, latitude] (GeoJSON order).
    #[serde(default)]
    pub coordinates: Vec<f64>,
}

/// Properties for a projected point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomProjectedProperties {
    /// Route index.
    #[serde(default, rename = "routeIndex")]
    pub route_index: Option<u32>,
    /// Snap result: "Matched", "OffRoad", or "MaxDistanceExceeded".
    #[serde(default, rename = "snapResult")]
    pub snap_result: Option<String>,
}

/// Distance summary from TomTom Snap to Roads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomDistances {
    /// Total distance.
    #[serde(default)]
    pub total: Option<f64>,
    /// Distance unit.
    #[serde(default)]
    pub unit: Option<String>,
}

impl From<TomTomProjectedPoint> for MatchedPoint {
    fn from(pp: TomTomProjectedPoint) -> Self {
        let coordinate = pp.geometry
            .and_then(|g| {
                // GeoJSON coordinates are [longitude, latitude]
                if g.coordinates.len() >= 2 {
                    Coordinate::new(g.coordinates[1], g.coordinates[0]).ok()
                } else {
                    None
                }
            })
            .unwrap_or(Coordinate::ORIGIN);

        MatchedPoint {
            coordinate,
            confidence: None,
            road_name: None,
        }
    }
}