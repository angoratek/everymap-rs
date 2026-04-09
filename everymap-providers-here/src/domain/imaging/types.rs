use serde::{Deserialize, Serialize};

/// Image format for map tile rendering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ImageFormat {
    #[default]
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpg")]
    Jpg,
    #[serde(rename = "gif")]
    Gif,
    #[serde(rename = "bmp")]
    Bmp,
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "png8")]
    Png8,
    #[serde(rename = "png32")]
    Png32,
}

/// Map tile style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum MapStyle {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "explore.day")]
    ExploreDay,
    #[serde(rename = "explore.night")]
    ExploreNight,
    #[serde(rename = "topo.day")]
    TopoDay,
    #[serde(rename = "topo.night")]
    TopoNight,
    #[serde(rename = "terrain.day")]
    TerrainDay,
    #[serde(rename = "satellite.day")]
    SatelliteDay,
    #[serde(rename = "hybrid.day")]
    HybridDay,
    #[serde(rename = "lc.day")]
    LcDay,
    #[serde(rename = "traffic.day")]
    TrafficDay,
}