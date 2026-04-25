use everymap_core::types::Coordinate;
use serde::{Deserialize, Serialize};

/// Geographic coordinate pair from HERE API responses.
///
/// This is a shared type used across multiple HERE API domains
/// (search, routing, traffic, etc.) for deserializing lat/lng pairs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereLatLng {
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lng: f64,
}

impl From<HereLatLng> for Coordinate {
    fn from(value: HereLatLng) -> Self {
        Coordinate::new(value.lat, value.lng).unwrap_or(Coordinate::ORIGIN)
    }
}
