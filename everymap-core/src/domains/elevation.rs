use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// One elevation sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElevationSample {
    /// The sampled position, echoed from the request.
    pub coordinate: Coordinate,
    /// The elevation above sea level, in meters.
    pub elevation_meters: f64,
}

/// Elevation sampling for ordered points.
#[async_trait]
pub trait ElevationProvider: Send + Sync {
    /// Fetches elevation samples for ordered points.
    ///
    /// # Parameters
    /// - `points` — the coordinates to sample, in order.
    ///
    /// # Returns
    /// One sample per requested point, in the same order.
    ///
    /// # Errors
    /// Returns [`crate::error::EveryMapError::ValidationError`] when no point
    /// is supplied, [`crate::error::EveryMapError::UnsupportedDomain`] when
    /// the provider cannot sample elevation, and
    /// [`crate::error::EveryMapError::SerializationError`] when the response
    /// cannot be decoded.
    async fn elevation_points(&self, points: &[Coordinate])
        -> EveryMapResult<Vec<ElevationSample>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_sample_roundtrip() {
        let samples = vec![ElevationSample {
            coordinate: Coordinate::new(38.5816, -121.4944).unwrap(),
            elevation_meters: 9.0,
        }];
        let json = serde_json::to_string(&samples).unwrap();
        let back: Vec<ElevationSample> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].elevation_meters, 9.0);
    }
}
