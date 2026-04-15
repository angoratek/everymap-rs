use serde::{Deserialize, Serialize};

/// Response from MapBox Isochrone API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxIsochroneResponse {
    #[serde(default)]
    pub features: Vec<MapBoxIsochroneFeature>,
}

/// A single isochrone contour from MapBox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxIsochroneFeature {
    #[serde(rename = "type", default)]
    pub feature_type: Option<String>,
    /// Properties with contour info.
    #[serde(default)]
    pub properties: Option<MapBoxIsochroneProperties>,
    /// Geometry of the isochrone polygon.
    #[serde(default)]
    pub geometry: Option<MapBoxIsochroneGeometry>,
}

/// Isochrone properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxIsochroneProperties {
    /// Contour time in minutes (for time-based isochrones).
    #[serde(default, rename = "contour")]
    pub contour: Option<u64>,
    /// Color index.
    #[serde(default, rename = "color")]
    pub color: Option<String>,
}

/// Isochrone geometry (Polygon).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxIsochroneGeometry {
    #[serde(rename = "type", default)]
    pub geometry_type: Option<String>,
    /// Polygon coordinates (nested array).
    #[serde(default)]
    pub coordinates: Option<serde_json::Value>,
}