use serde::{Deserialize, Serialize};

/// Radar-specific lat/lng coordinate type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RadarLatLng {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}