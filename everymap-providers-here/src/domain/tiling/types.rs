use serde::{Deserialize, Serialize};

/// Available vector tile layers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TileLayer {
    #[default]
    #[serde(rename = "mapbox")]
    Mapbox,
    #[serde(rename = "base")]
    Base,
    #[serde(rename = "core")]
    Core,
    #[serde(rename = "hybrid")]
    Hybrid,
}

/// Tile format for the response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TileFormat {
    #[default]
    #[serde(rename = "omv")]
    OmnichannelVector,
    #[serde(rename = "pbf")]
    Protobuf,
}