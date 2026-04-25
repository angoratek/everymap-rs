use crate::domain::types::{RadarLocation, RadarMeta};
use serde::{Deserialize, Serialize};

/// Response from Radar trip creation/update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTripCreateResponse {
    pub meta: RadarMeta,
    pub trip: RadarTrip,
}

/// Response from Radar trip get.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTripGetResponse {
    pub meta: RadarMeta,
    pub trip: RadarTrip,
}

/// A Radar trip object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTrip {
    #[serde(default, rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    pub origin: Option<RadarLocation>,
    pub destination: Option<RadarLocation>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub eta: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}
