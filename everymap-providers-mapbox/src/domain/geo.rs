use serde::{Deserialize, Serialize};

/// MapBox latitude/longitude coordinate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxLatLng {
    pub lat: f64,
    pub lng: f64,
}

/// MapBox bounding box: [west, south, east, north].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapBoxBounds {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}
