use serde::{Deserialize, Serialize};
use super::coordinate::Coordinate;

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
