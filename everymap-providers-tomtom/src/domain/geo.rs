use serde::{Deserialize, Serialize};

/// A latitude/longitude pair from TomTom APIs.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TomTomLatLng {
    /// Latitude in degrees.
    #[serde(default)]
    pub lat: f64,
    /// Longitude in degrees.
    #[serde(default)]
    pub lon: f64,
}

/// A bounding box from TomTom APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomBounds {
    /// Top-left corner.
    #[serde(default)]
    pub top_left: TomTomLatLng,
    /// Bottom-right corner.
    #[serde(default)]
    pub btm_right: TomTomLatLng,
}