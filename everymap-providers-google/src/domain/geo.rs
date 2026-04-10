use serde::{Deserialize, Serialize};

/// Shared lat/lng coordinate type for Google provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleLatLng {
    pub lat: f64,
    pub lng: f64,
}

/// Bounds (northeast/southwest rectangle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleBounds {
    pub northeast: GoogleLatLng,
    pub southwest: GoogleLatLng,
}