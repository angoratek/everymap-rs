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

/// A geocoding feature from MapBox v6.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxFeature {
    #[serde(rename = "type", default)]
    pub feature_type: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    /// Geometry with coordinates.
    #[serde(default, rename = "geometry")]
    pub geometry: Option<MapBoxGeometry>,
    /// Properties containing name, address, coordinates, bbox, context.
    #[serde(default, rename = "properties")]
    pub properties: Option<MapBoxProperties>,
}

/// MapBox v6 feature properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxProperties {
    /// MapBox internal ID.
    #[serde(default, rename = "mapbox_id")]
    pub mapbox_id: Option<String>,
    /// Feature type (e.g., "place", "address", "poi", "region", "country").
    #[serde(default, rename = "feature_type")]
    pub feature_type: Option<String>,
    /// Full address string (e.g., "Berlin, Germany").
    #[serde(default, rename = "full_address")]
    pub full_address: Option<String>,
    /// Primary name of the feature.
    #[serde(default)]
    pub name: Option<String>,
    /// Preferred name variant.
    #[serde(default, rename = "name_preferred")]
    pub name_preferred: Option<String>,
    /// Formatted place context (e.g., "Germany" or "New Hampshire, United States").
    #[serde(default, rename = "place_formatted")]
    pub place_formatted: Option<String>,
    /// Coordinate object with longitude/latitude.
    #[serde(default, rename = "coordinates")]
    pub coordinates: Option<MapBoxCoordinates>,
    /// Bounding box as [west, south, east, north] inside properties.
    #[serde(default, rename = "bbox")]
    pub bbox: Option<Vec<f64>>,
    /// Context with region, country, district, place info.
    #[serde(default)]
    pub context: Option<MapBoxContext>,
    /// Additional feature types.
    #[serde(default, rename = "additional_feature_types")]
    pub additional_feature_types: Option<Vec<String>>,
    /// Address (house number) for address-type features.
    #[serde(default)]
    pub address: Option<String>,
    /// Category for POI features.
    #[serde(default)]
    pub category: Option<String>,
    /// Relevance score (0.0 to 1.0).
    #[serde(default)]
    pub relevance: Option<f64>,
}

/// Coordinate object from MapBox v6 properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxCoordinates {
    #[serde(default)]
    pub longitude: f64,
    #[serde(default)]
    pub latitude: f64,
}

/// Context information from MapBox v6.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxContext {
    #[serde(default)]
    pub region: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub country: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub district: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub place: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub neighborhood: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub street: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub postcode: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub address: Option<MapBoxContextEntry>,
    #[serde(default)]
    pub poi: Option<MapBoxContextEntry>,
}

/// A context entry (region, country, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxContextEntry {
    #[serde(default, rename = "mapbox_id")]
    pub mapbox_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "region_code")]
    pub region_code: Option<String>,
    #[serde(default, rename = "country_code")]
    pub country_code: Option<String>,
    #[serde(default, rename = "country_code_alpha_3")]
    pub country_code_alpha_3: Option<String>,
    #[serde(default)]
    pub wikidata_id: Option<String>,
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
