use crate::domain::types::RadarMeta;
use serde::{Deserialize, Serialize};

/// Response from Radar Track API (used for fraud detection).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTrackResponse {
    pub meta: RadarMeta,
    pub user: RadarTrackUser,
    #[serde(default)]
    pub events: Vec<serde_json::Value>,
}

/// User object from Radar Track response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarTrackUser {
    #[serde(default)]
    pub fraud: Option<RadarFraudData>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
}

/// Fraud detection data from Radar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarFraudData {
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub passed: bool,
    #[serde(default)]
    pub mocked: bool,
    #[serde(default)]
    pub jumped: bool,
    #[serde(default)]
    pub compromised: bool,
    #[serde(default)]
    pub inaccurate: bool,
    #[serde(default)]
    pub proxy: bool,
    #[serde(default)]
    pub sharing: bool,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub bypassed: bool,
}
