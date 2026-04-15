use serde::{Deserialize, Serialize};

/// Response from MapBox Geocoding API v6.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxSearchResponse {
    #[serde(rename = "type", default)]
    pub response_type: Option<String>,
    #[serde(default)]
    pub features: Vec<MapBoxFeature>,
    #[serde(default, rename = "attribution")]
    pub attribution: Option<String>,
}

/// A geocoding feature from MapBox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxFeature {
    #[serde(rename = "type", default)]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "place_type")]
    pub place_type: Vec<String>,
    #[serde(default, rename = "relevance")]
    pub relevance: Option<f64>,
    #[serde(default, rename = "properties")]
    pub properties: Option<MapBoxProperties>,
    #[serde(default, rename = "text")]
    pub text: Option<String>,
    #[serde(default, rename = "place_name")]
    pub place_name: Option<String>,
    /// Center coordinate as [lng, lat].
    #[serde(default, rename = "center")]
    pub center: Option<Vec<f64>>,
    /// Geometry coordinate as [lng, lat] (point) or nested array (polygon).
    #[serde(default, rename = "geometry")]
    pub geometry: Option<MapBoxGeometry>,
    /// Bounding box as [west, south, east, north].
    #[serde(default, rename = "bbox")]
    pub bbox: Option<Vec<f64>>,
}

/// MapBox feature properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxProperties {
    #[serde(default, rename = "feature_type")]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default, rename = "category")]
    pub category: Option<String>,
}

/// MapBox geometry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxGeometry {
    #[serde(rename = "type", default)]
    pub geometry_type: Option<String>,
    /// Coordinates as [lng, lat] for Point, or nested for Polygon.
    #[serde(default)]
    pub coordinates: Option<serde_json::Value>,
}