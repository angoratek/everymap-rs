use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a geospatial coordinate with latitude and longitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

impl Coordinate {
    /// The origin coordinate (0°, 0°), also known as "Null Island".
    /// Used as a fallback when real coordinate data is missing.
    pub const ORIGIN: Self = Self { lat: 0.0, lng: 0.0 };

    pub fn new(lat: f64, lng: f64) -> Result<Self, CoordinateError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(CoordinateError::InvalidLatitude(lat));
        }
        if !(-180.0..=180.0).contains(&lng) {
            return Err(CoordinateError::InvalidLongitude(lng));
        }
        Ok(Self { lat, lng })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoordinateError {
    #[error("Invalid latitude: {0}. Must be between -90 and 90.")]
    InvalidLatitude(f64),
    #[error("Invalid longitude: {0}. Must be between -180 and 180.")]
    InvalidLongitude(f64),
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.lat, self.lng)
    }
}
