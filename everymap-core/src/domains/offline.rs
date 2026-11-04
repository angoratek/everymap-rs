use crate::domains::routing::{RouteResponse, TransportMode};
use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Where an offline routing provider reads its graph from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OfflineGraphSource {
    /// An in-memory graph archive.
    ArchiveBytes(Vec<u8>),
    /// A filesystem path the target platform can read.
    FilePath(String),
}

/// One offline route request over a loaded graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineRouteRequest {
    /// The ordered waypoints, starting point first.
    pub waypoints: Vec<Coordinate>,
    /// The mode to route for.
    pub transport_mode: TransportMode,
}

/// Routing over a graph compiled for offline use.
///
/// The trait is transport-only: an implementation owns its graph format and
/// validation rules, so a caller can swap engines without changing its own
/// code. A provider that has no offline graph returns
/// [`crate::error::EveryMapError::UnsupportedDomain`].
#[async_trait]
pub trait OfflineRoutingProvider: Send + Sync {
    /// Loads and validates an offline graph.
    ///
    /// # Parameters
    /// - `source` — the archive bytes or a filesystem path.
    ///
    /// # Returns
    /// `Ok(())` once the graph is validated and ready for routing.
    ///
    /// # Errors
    /// Returns [`crate::error::EveryMapError::ValidationError`] when the
    /// archive is malformed, truncated or of an unsupported version.
    async fn load_offline_graph(&self, source: OfflineGraphSource) -> EveryMapResult<()>;

    /// Calculates a route over the loaded graph.
    ///
    /// # Parameters
    /// - `request` — the ordered waypoints and the transport mode.
    ///
    /// # Returns
    /// The navigation-grade route response, the same shape a live router
    /// returns.
    ///
    /// # Errors
    /// Returns [`crate::error::EveryMapError::ValidationError`] when fewer
    /// than two waypoints are supplied or a waypoint lies outside the graph,
    /// and [`crate::error::EveryMapError::UnsupportedDomain`] when the graph
    /// cannot route the requested mode.
    async fn offline_route(&self, request: &OfflineRouteRequest) -> EveryMapResult<RouteResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_route_request_roundtrip() {
        let request = OfflineRouteRequest {
            waypoints: vec![
                Coordinate::new(38.58, -121.49).unwrap(),
                Coordinate::new(38.59, -121.48).unwrap(),
            ],
            transport_mode: TransportMode::Pedestrian,
        };
        let json = serde_json::to_string(&request).unwrap();
        let back: OfflineRouteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.waypoints.len(), 2);
        assert_eq!(back.transport_mode, TransportMode::Pedestrian);
    }

    #[test]
    fn test_offline_graph_source_path_roundtrip() {
        let source = OfflineGraphSource::FilePath("/graphs/sacramento.rmog".to_string());
        let json = serde_json::to_string(&source).unwrap();
        let back: OfflineGraphSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source);
    }
}
