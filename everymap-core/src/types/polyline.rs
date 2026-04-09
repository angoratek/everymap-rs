use super::coordinate::Coordinate;

/// Flexible Polyline encoding/decoding using the HERE flexible polyline format.
///
/// This wraps the `flexpolyline` crate which implements the official HERE
/// Flexible Polyline Algorithm for lossy compression of coordinate sequences.
pub struct FlexiblePolyline;

impl FlexiblePolyline {
    /// Decodes a HERE flexible polyline string into a vector of Coordinates.
    ///
    /// The polyline format uses a configurable precision and optional third dimension
    /// (e.g., elevation). This method decodes 2D polylines (lat/lng only).
    pub fn decode(encoded: &str) -> Result<Vec<Coordinate>, String> {
        if encoded.is_empty() {
            return Ok(vec![]);
        }

        let decoded = flexpolyline::Polyline::decode(encoded)
            .map_err(|e| format!("Failed to decode flexible polyline: {:?}", e))?;

        match decoded {
            flexpolyline::Polyline::Data2d { coordinates, .. } => {
                coordinates
                    .into_iter()
                    .map(|(lat, lng)| {
                        Coordinate::new(lat, lng)
                            .map_err(|e| format!("Invalid coordinate: {}", e))
                    })
                    .collect()
            }
            flexpolyline::Polyline::Data3d { coordinates, .. } => {
                coordinates
                    .into_iter()
                    .map(|(lat, lng, _)| {
                        Coordinate::new(lat, lng)
                            .map_err(|e| format!("Invalid coordinate: {}", e))
                    })
                    .collect()
            }
        }
    }

    /// Encodes a vector of Coordinates into a HERE flexible polyline string
    /// with 5 digits of precision (the default for most HERE APIs).
    pub fn encode(coordinates: &[Coordinate]) -> Result<String, String> {
        if coordinates.is_empty() {
            return Ok(String::new());
        }

        let coords: Vec<(f64, f64)> = coordinates
            .iter()
            .map(|c| (c.lat, c.lng))
            .collect();

        let polyline = flexpolyline::Polyline::Data2d {
            coordinates: coords,
            precision2d: flexpolyline::Precision::Digits5,
        };

        polyline.encode()
            .map_err(|e| format!("Failed to encode flexible polyline: {:?}", e))
    }

    /// Encodes with a custom precision level.
    pub fn encode_with_precision(coordinates: &[Coordinate], precision: flexpolyline::Precision) -> Result<String, String> {
        if coordinates.is_empty() {
            return Ok(String::new());
        }

        let coords: Vec<(f64, f64)> = coordinates
            .iter()
            .map(|c| (c.lat, c.lng))
            .collect();

        let polyline = flexpolyline::Polyline::Data2d {
            coordinates: coords,
            precision2d: precision,
        };

        polyline.encode()
            .map_err(|e| format!("Failed to encode flexible polyline: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_string() {
        let result = FlexiblePolyline::decode("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_encode_empty_coords() {
        let result = FlexiblePolyline::encode(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_roundtrip_encode_decode() {
        let coords = vec![
            Coordinate::new(50.1022829, 8.6982122).unwrap(),
            Coordinate::new(50.1020076, 8.6956695).unwrap(),
            Coordinate::new(50.1006313, 8.6914960).unwrap(),
            Coordinate::new(50.0987800, 8.6875156).unwrap(),
        ];

        let encoded = FlexiblePolyline::encode(&coords).unwrap();
        // The flexible polyline encoding rounds to the specified precision
        assert!(!encoded.is_empty());

        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), coords.len());

        // Check that coordinates are approximately equal (within precision)
        for (original, decoded) in coords.iter().zip(decoded.iter()) {
            assert!((original.lat - decoded.lat).abs() < 0.00001);
            assert!((original.lng - decoded.lng).abs() < 0.00001);
        }
    }

    #[test]
    fn test_decode_known_polyline() {
        // This is the official test vector from the HERE flexible-polyline spec
        let encoded = "BFoz5xJ67i1B1B7PzIhaxL7Y";
        let decoded = FlexiblePolyline::decode(encoded).unwrap();

        assert_eq!(decoded.len(), 4);
        // The first point should be approximately (50.10228, 8.69821)
        assert!((decoded[0].lat - 50.10228).abs() < 0.001);
        assert!((decoded[0].lng - 8.69821).abs() < 0.001);
    }
}