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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Boundary values (should succeed) ---

    #[test]
    fn test_lat_90_boundary() {
        let c = Coordinate::new(90.0, 0.0);
        assert!(c.is_ok());
        assert_eq!(c.unwrap().lat, 90.0);
    }

    #[test]
    fn test_lat_neg_90_boundary() {
        let c = Coordinate::new(-90.0, 0.0);
        assert!(c.is_ok());
        assert_eq!(c.unwrap().lat, -90.0);
    }

    #[test]
    fn test_lng_180_boundary() {
        let c = Coordinate::new(0.0, 180.0);
        assert!(c.is_ok());
        assert_eq!(c.unwrap().lng, 180.0);
    }

    #[test]
    fn test_lng_neg_180_boundary() {
        let c = Coordinate::new(0.0, -180.0);
        assert!(c.is_ok());
        assert_eq!(c.unwrap().lng, -180.0);
    }

    // --- Out-of-range values (should fail) ---

    #[test]
    fn test_lat_91_out_of_range() {
        let c = Coordinate::new(91.0, 0.0);
        assert!(c.is_err());
        match c.unwrap_err() {
            CoordinateError::InvalidLatitude(v) => assert_eq!(v, 91.0),
            _ => panic!("Expected InvalidLatitude"),
        }
    }

    #[test]
    fn test_lat_neg_91_out_of_range() {
        let c = Coordinate::new(-91.0, 0.0);
        assert!(c.is_err());
        match c.unwrap_err() {
            CoordinateError::InvalidLatitude(v) => assert_eq!(v, -91.0),
            _ => panic!("Expected InvalidLatitude"),
        }
    }

    #[test]
    fn test_lng_181_out_of_range() {
        let c = Coordinate::new(0.0, 181.0);
        assert!(c.is_err());
        match c.unwrap_err() {
            CoordinateError::InvalidLongitude(v) => assert_eq!(v, 181.0),
            _ => panic!("Expected InvalidLongitude"),
        }
    }

    #[test]
    fn test_lng_neg_181_out_of_range() {
        let c = Coordinate::new(0.0, -181.0);
        assert!(c.is_err());
        match c.unwrap_err() {
            CoordinateError::InvalidLongitude(v) => assert_eq!(v, -181.0),
            _ => panic!("Expected InvalidLongitude"),
        }
    }

    // --- Zero values ---

    #[test]
    fn test_zero_zero() {
        let c = Coordinate::new(0.0, 0.0).unwrap();
        assert_eq!(c.lat, 0.0);
        assert_eq!(c.lng, 0.0);
    }

    // --- NaN and Infinity (should fail) ---

    #[test]
    fn test_lat_nan_fails() {
        let c = Coordinate::new(f64::NAN, 0.0);
        assert!(c.is_err());
    }

    #[test]
    fn test_lng_nan_fails() {
        let c = Coordinate::new(0.0, f64::NAN);
        assert!(c.is_err());
    }

    #[test]
    fn test_lat_infinity_fails() {
        let c = Coordinate::new(f64::INFINITY, 0.0);
        assert!(c.is_err());
    }

    #[test]
    fn test_lat_neg_infinity_fails() {
        let c = Coordinate::new(f64::NEG_INFINITY, 0.0);
        assert!(c.is_err());
    }

    #[test]
    fn test_lng_infinity_fails() {
        let c = Coordinate::new(0.0, f64::INFINITY);
        assert!(c.is_err());
    }

    #[test]
    fn test_lng_neg_infinity_fails() {
        let c = Coordinate::new(0.0, f64::NEG_INFINITY);
        assert!(c.is_err());
    }

    // --- Display format ---

    #[test]
    fn test_display_format() {
        let c = Coordinate::new(52.5, 13.4).unwrap();
        assert_eq!(format!("{}", c), "(52.5, 13.4)");
    }

    #[test]
    fn test_display_format_negative() {
        let c = Coordinate::new(-33.8688, 151.2093).unwrap();
        assert_eq!(format!("{}", c), "(-33.8688, 151.2093)");
    }

    #[test]
    fn test_display_format_zero() {
        let c = Coordinate::new(0.0, 0.0).unwrap();
        assert_eq!(format!("{}", c), "(0, 0)");
    }

    // --- Serde roundtrip ---

    #[test]
    fn test_serde_roundtrip() {
        let c = Coordinate::new(52.5, 13.4).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Coordinate = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_boundary() {
        let c = Coordinate::new(90.0, -180.0).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Coordinate = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_negative() {
        let c = Coordinate::new(-45.0, -90.0).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Coordinate = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn test_serde_json_structure() {
        let c = Coordinate::new(52.5, 13.4).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"lat\""));
        assert!(json.contains("\"lng\""));
    }

    // --- ORIGIN constant ---

    #[test]
    fn test_origin_constant() {
        assert_eq!(Coordinate::ORIGIN.lat, 0.0);
        assert_eq!(Coordinate::ORIGIN.lng, 0.0);
    }

    #[test]
    fn test_origin_equals_new_zero() {
        let zero = Coordinate::new(0.0, 0.0).unwrap();
        assert_eq!(Coordinate::ORIGIN, zero);
    }

    // --- Copy and Clone ---

    #[test]
    fn test_coordinate_copy() {
        let c = Coordinate::new(52.5, 13.4).unwrap();
        let c2 = c;
        assert_eq!(c, c2);
        // c is still valid because it's Copy
        assert_eq!(c.lat, 52.5);
    }

    // --- Partial equality ---

    #[test]
    fn test_coordinate_equality() {
        let c1 = Coordinate::new(52.5, 13.4).unwrap();
        let c2 = Coordinate::new(52.5, 13.4).unwrap();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_coordinate_inequality() {
        let c1 = Coordinate::new(52.5, 13.4).unwrap();
        let c2 = Coordinate::new(52.6, 13.4).unwrap();
        assert_ne!(c1, c2);
    }

    // --- Error display format ---

    #[test]
    fn test_coordinate_error_display() {
        let err = CoordinateError::InvalidLatitude(91.0);
        let msg = format!("{}", err);
        assert!(msg.contains("91"));
        assert!(msg.contains("latitude"));

        let err = CoordinateError::InvalidLongitude(200.0);
        let msg = format!("{}", err);
        assert!(msg.contains("200"));
        assert!(msg.contains("longitude"));
    }

    // --- Both lat and lng invalid: lat checked first ---

    #[test]
    fn test_both_invalid_lat_checked_first() {
        let c = Coordinate::new(91.0, 181.0);
        assert!(c.is_err());
        match c.unwrap_err() {
            CoordinateError::InvalidLatitude(_) => {}
            CoordinateError::InvalidLongitude(_) => panic!("Should check latitude first"),
        }
    }
}
