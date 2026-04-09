use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for tour/sequence optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourRequest<O> {
    pub stops: Vec<Coordinate>,
    pub options: O,
}

/// A stop in an optimized tour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourStop {
    /// The coordinate of this stop
    pub coordinate: Coordinate,
    /// Arrival time (ISO 8601 string, if available)
    pub arrival_time: Option<String>,
    /// Departure time (ISO 8601 string, if available)
    pub departure_time: Option<String>,
    /// Duration at this stop in seconds
    pub duration: Option<f64>,
    /// Distance from previous stop in meters
    pub distance_from_previous: Option<f64>,
}

/// A unified tour response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourResponse {
    /// The optimized sequence of stops
    pub stops: Vec<TourStop>,
    /// Total tour distance in meters
    pub total_distance: Option<f64>,
    /// Total tour duration in seconds
    pub total_duration: Option<f64>,
    /// Number of unassigned stops (that couldn't be fit into the tour)
    pub unassigned_count: Option<u32>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait TourPlanner: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn optimize_tour(&self, req: TourRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}