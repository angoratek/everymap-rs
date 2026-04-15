use serde::{Deserialize, Serialize};
use crate::domain::types::{RadarMeta, RadarLocation};

/// Response from Radar geofence search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarGeofenceSearchResponse {
    pub meta: RadarMeta,
    #[serde(default)]
    pub geofences: Vec<RadarGeofence>,
}

/// Response from Radar geofence creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarGeofenceCreateResponse {
    pub meta: RadarMeta,
    pub geofence: RadarGeofence,
}

/// Response from Radar geofence get.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarGeofenceGetResponse {
    pub meta: RadarMeta,
    pub geofence: RadarGeofence,
}

/// A Radar geofence object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarGeofence {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub live: Option<bool>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub gf_type: Option<String>,
    pub geometry_center: Option<RadarLocation>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub radius: Option<f64>,
}