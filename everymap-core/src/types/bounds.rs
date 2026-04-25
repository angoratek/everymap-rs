use super::coordinate::Coordinate;
use serde::{Deserialize, Serialize};

/// Represents a geospatial bounding box defined by two coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub north_east: Coordinate,
    pub south_west: Coordinate,
}

impl BoundingBox {
    pub fn new(north_east: Coordinate, south_west: Coordinate) -> Self {
        Self {
            north_east,
            south_west,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    pub points: Vec<Coordinate>,
}

impl Polyline {
    pub fn new(points: Vec<Coordinate>) -> Self {
        Self { points }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BoundingBox tests ---

    #[test]
    fn test_bounding_box_construction() {
        let ne = Coordinate::new(52.5, 13.5).unwrap();
        let sw = Coordinate::new(52.0, 13.0).unwrap();
        let bbox = BoundingBox::new(ne, sw);
        assert_eq!(bbox.north_east.lat, 52.5);
        assert_eq!(bbox.north_east.lng, 13.5);
        assert_eq!(bbox.south_west.lat, 52.0);
        assert_eq!(bbox.south_west.lng, 13.0);
    }

    #[test]
    fn test_bounding_box_serde_roundtrip() {
        let ne = Coordinate::new(52.5, 13.5).unwrap();
        let sw = Coordinate::new(52.0, 13.0).unwrap();
        let bbox = BoundingBox::new(ne, sw);
        let json = serde_json::to_string(&bbox).unwrap();
        let back: BoundingBox = serde_json::from_str(&json).unwrap();
        assert_eq!(bbox, back);
    }

    #[test]
    fn test_bounding_box_equality() {
        let ne = Coordinate::new(52.5, 13.5).unwrap();
        let sw = Coordinate::new(52.0, 13.0).unwrap();
        let b1 = BoundingBox::new(ne, sw);
        let b2 = BoundingBox::new(ne, sw);
        assert_eq!(b1, b2);
    }

    #[test]
    fn test_bounding_box_copy() {
        let ne = Coordinate::new(52.5, 13.5).unwrap();
        let sw = Coordinate::new(52.0, 13.0).unwrap();
        let b1 = BoundingBox::new(ne, sw);
        let b2 = b1;
        assert_eq!(b1, b2);
        // b1 is still valid because BoundingBox is Copy
        assert_eq!(b1.north_east.lat, 52.5);
    }

    #[test]
    fn test_bounding_box_boundary_coordinates() {
        let ne = Coordinate::new(90.0, 180.0).unwrap();
        let sw = Coordinate::new(-90.0, -180.0).unwrap();
        let bbox = BoundingBox::new(ne, sw);
        assert_eq!(bbox.north_east.lat, 90.0);
        assert_eq!(bbox.south_west.lat, -90.0);
    }

    // --- Polyline tests ---

    #[test]
    fn test_polyline_new_empty() {
        let p = Polyline::new(vec![]);
        assert!(p.points.is_empty());
    }

    #[test]
    fn test_polyline_new_with_points() {
        let points = vec![
            Coordinate::new(52.5, 13.4).unwrap(),
            Coordinate::new(52.6, 13.5).unwrap(),
        ];
        let p = Polyline::new(points);
        assert_eq!(p.points.len(), 2);
        assert_eq!(p.points[0].lat, 52.5);
        assert_eq!(p.points[1].lat, 52.6);
    }

    #[test]
    fn test_polyline_serde_roundtrip() {
        let points = vec![
            Coordinate::new(52.5, 13.4).unwrap(),
            Coordinate::new(52.6, 13.5).unwrap(),
        ];
        let p = Polyline::new(points);
        let json = serde_json::to_string(&p).unwrap();
        let back: Polyline = serde_json::from_str(&json).unwrap();
        assert_eq!(back.points.len(), 2);
        assert_eq!(back.points[0], p.points[0]);
        assert_eq!(back.points[1], p.points[1]);
    }

    #[test]
    fn test_polyline_with_many_points() {
        let points: Vec<Coordinate> = (0..100)
            .map(|i| Coordinate::new(i as f64 * 0.1, i as f64 * 0.2).unwrap())
            .collect();
        let p = Polyline::new(points);
        assert_eq!(p.points.len(), 100);
        let json = serde_json::to_string(&p).unwrap();
        let back: Polyline = serde_json::from_str(&json).unwrap();
        assert_eq!(back.points.len(), 100);
        assert_eq!(back.points[0].lat, 0.0);
        assert_eq!(back.points[99].lat, 9.9);
    }

    #[test]
    fn test_polyline_equality() {
        let p1 = Polyline::new(vec![Coordinate::new(1.0, 2.0).unwrap()]);
        let p2 = Polyline::new(vec![Coordinate::new(1.0, 2.0).unwrap()]);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_polyline_inequality() {
        let p1 = Polyline::new(vec![Coordinate::new(1.0, 2.0).unwrap()]);
        let p2 = Polyline::new(vec![Coordinate::new(1.0, 3.0).unwrap()]);
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_polyline_clone() {
        let p = Polyline::new(vec![Coordinate::new(1.0, 2.0).unwrap()]);
        let cloned = p.clone();
        assert_eq!(p, cloned);
    }
}
