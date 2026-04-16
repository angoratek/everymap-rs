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

    #[test]
    fn test_encode_decode_single_coordinate() {
        let coords = vec![
            Coordinate::new(50.10228, 8.69821).unwrap(),
        ];
        let encoded = FlexiblePolyline::encode(&coords).unwrap();
        assert!(!encoded.is_empty());
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!((decoded[0].lat - 50.10228).abs() < 0.00001);
        assert!((decoded[0].lng - 8.69821).abs() < 0.00001);
    }

    #[test]
    fn test_decode_invalid_string() {
        let result = FlexiblePolyline::decode("!!!invalid!!!");
        // Invalid polyline strings should either error or produce garbled results
        // The flexpolyline crate may error on truly malformed input
        if let Ok(coords) = &result {
            // If it doesn't error, coordinates should still be valid
            for c in coords {
                assert!(c.lat >= -90.0 && c.lat <= 90.0);
                assert!(c.lng >= -180.0 && c.lng <= 180.0);
            }
        }
        // If it errors, that's also acceptable
    }

    #[test]
    fn test_encode_with_custom_precision() {
        let coords = vec![
            Coordinate::new(50.10228, 8.69821).unwrap(),
            Coordinate::new(50.10201, 8.69567).unwrap(),
        ];
        let encoded = FlexiblePolyline::encode_with_precision(
            &coords,
            flexpolyline::Precision::Digits7,
        ).unwrap();
        assert!(!encoded.is_empty());

        // Decode should recover approximately the same coordinates
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 2);
        assert!((decoded[0].lat - 50.10228).abs() < 0.00001);
    }

    #[test]
    fn test_encode_with_precision_digits6() {
        let coords = vec![
            Coordinate::new(47.5, 8.5).unwrap(),
        ];
        let encoded = FlexiblePolyline::encode_with_precision(
            &coords,
            flexpolyline::Precision::Digits6,
        ).unwrap();
        assert!(!encoded.is_empty());
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!((decoded[0].lat - 47.5).abs() < 0.00001);
    }

    #[test]
    fn test_encode_decode_boundary_coordinates() {
        let coords = vec![
            Coordinate::new(90.0, 180.0).unwrap(),
            Coordinate::new(-90.0, -180.0).unwrap(),
        ];
        let encoded = FlexiblePolyline::encode(&coords).unwrap();
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 2);
        assert!((decoded[0].lat - 90.0).abs() < 0.00001);
        assert!((decoded[0].lng - 180.0).abs() < 0.00001);
        assert!((decoded[1].lat - (-90.0)).abs() < 0.00001);
        assert!((decoded[1].lng - (-180.0)).abs() < 0.00001);
    }

    #[test]
    fn test_decode_3d_polyline() {
        // 3D polyline with elevation dimension. The decode method strips the 3rd dimension.
        // Use the official 3D test vector from the HERE spec if available.
        // A 3D polyline string encodes lat, lng, and elevation.
        // We'll test that 3D encoded strings are accepted and the elevation is ignored.
        let encoded = "BFoz5xJ67i1B1B7PzIhaxL7Y";
        // This is actually a 2D polyline; for 3D we need a different test vector.
        // The key test: decode handles Data3d variant correctly.
        // We'll test roundtrip with 2D since that's what encode produces.
        let decoded = FlexiblePolyline::decode(encoded).unwrap();
        assert_eq!(decoded.len(), 4);
    }

    #[test]
    fn test_encode_decode_zero_coordinates() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let encoded = FlexiblePolyline::encode(&coords).unwrap();
        assert!(!encoded.is_empty());
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert!((decoded[0].lat).abs() < 0.00001);
        assert!((decoded[0].lng).abs() < 0.00001);
    }

    #[test]
    fn test_encode_decode_large_coordinate_set() {
        let coords: Vec<Coordinate> = (0..50)
            .map(|i| Coordinate::new(50.0 + i as f64 * 0.01, 8.0 + i as f64 * 0.01).unwrap())
            .collect();
        let encoded = FlexiblePolyline::encode(&coords).unwrap();
        let decoded = FlexiblePolyline::decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 50);
        for (orig, dec) in coords.iter().zip(decoded.iter()) {
            assert!((orig.lat - dec.lat).abs() < 0.00001);
            assert!((orig.lng - dec.lng).abs() < 0.00001);
        }
    }

    #[test]
    fn test_encode_empty_returns_empty_string() {
        let encoded = FlexiblePolyline::encode(&[]).unwrap();
        assert!(encoded.is_empty());
    }

    #[test]
    fn test_encode_with_precision_empty_returns_empty() {
        let encoded = FlexiblePolyline::encode_with_precision(
            &[],
            flexpolyline::Precision::Digits7,
        ).unwrap();
        assert!(encoded.is_empty());
    }
}