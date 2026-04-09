use serde::{Deserialize, Serialize};

/// Available attribute layers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AttributeLayer {
    #[default]
    #[serde(rename = "roads")]
    Roads,
    #[serde(rename = "adminAreas")]
    AdminAreas,
    #[serde(rename = "buildings")]
    Buildings,
    #[serde(rename = "landmarks")]
    Landmarks,
    #[serde(rename = "segments")]
    Segments,
}

/// Response format for attribute queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AttributeFormat {
    #[default]
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "geojson")]
    GeoJson,
    #[serde(rename = "protobuf")]
    Protobuf,
}