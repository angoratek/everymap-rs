use everymap_core::types::Coordinate;
use serde::{Deserialize, Serialize};

/// Geographic coordinate pair from HERE API responses.
///
/// This is a shared type used across multiple HERE API domains
/// (search, routing, traffic, etc.) for deserializing lat/lng pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereLatLng {
    pub lat: f64,
    pub lng: f64,
}

impl From<HereLatLng> for Coordinate {
    fn from(val: HereLatLng) -> Self {
        Coordinate::new(val.lat, val.lng).unwrap_or_else(|_| Coordinate::new(0.0, 0.0).unwrap())
    }
}