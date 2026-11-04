use crate::domains::routing::{RouteStrategy, TransportMode};
use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for an origin-destination distance matrix.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistanceMatrixOptions {
    /// The mode every pair is measured for.
    pub transport_mode: Option<TransportMode>,
    /// What the provider optimizes for, when it offers the choice.
    pub route_strategy: Option<RouteStrategy>,
    /// Provider-specific options for advanced use cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// One origin-destination measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMatrixEntry {
    /// Index into the request's origin list.
    pub origin_index: usize,
    /// Index into the request's destination list.
    pub destination_index: usize,
    /// Traveled distance in meters, when the provider reports it.
    pub distance_meters: Option<f64>,
    /// Travel time in seconds, when the provider reports it.
    pub duration_seconds: Option<f64>,
}

/// Every measured pair, in row-major origin-major order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMatrixResponse {
    /// One entry per origin-destination pair.
    pub entries: Vec<DistanceMatrixEntry>,
}

/// Duration and distance for every origin-destination pair.
#[async_trait]
pub trait DistanceMatrixProvider: Send + Sync {
    /// Calculates durations and distances for every origin-destination pair.
    ///
    /// # Parameters
    /// - `origins` — the origin coordinates.
    /// - `destinations` — the destination coordinates.
    /// - `options` — the transport mode, strategy and provider extras.
    ///
    /// # Returns
    /// One entry per pair, in row-major origin-major order.
    ///
    /// # Errors
    /// Returns [`crate::error::EveryMapError::ValidationError`] when a list is
    /// empty or a provider limit is exceeded,
    /// [`crate::error::EveryMapError::UnsupportedDomain`] when the provider
    /// cannot serve a matrix, and
    /// [`crate::error::EveryMapError::SerializationError`] when the response
    /// cannot be decoded.
    async fn distance_matrix(
        &self,
        origins: &[Coordinate],
        destinations: &[Coordinate],
        options: &DistanceMatrixOptions,
    ) -> EveryMapResult<DistanceMatrixResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_matrix_options_default() {
        let options = DistanceMatrixOptions::default();
        assert!(options.transport_mode.is_none());
        assert!(options.route_strategy.is_none());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_distance_matrix_entry_roundtrip() {
        let response = DistanceMatrixResponse {
            entries: vec![
                DistanceMatrixEntry {
                    origin_index: 0,
                    destination_index: 0,
                    distance_meters: Some(1500.0),
                    duration_seconds: Some(120.0),
                },
                DistanceMatrixEntry {
                    origin_index: 1,
                    destination_index: 0,
                    distance_meters: None,
                    duration_seconds: None,
                },
            ],
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: DistanceMatrixResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entries.len(), 2);
        assert_eq!(back.entries[0].distance_meters, Some(1500.0));
        assert!(back.entries[1].duration_seconds.is_none());
    }
}
