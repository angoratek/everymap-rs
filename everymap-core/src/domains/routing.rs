use async_trait::async_trait;
use crate::types::{Coordinate, Polyline, BoundingBox};
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for route calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest<O> {
    pub start: Coordinate,
    pub end: Coordinate,
    pub options: O,
}

/// A single leg/step in a route (e.g., a turn-by-turn instruction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    /// Instruction text (e.g., "Turn right onto Main St")
    pub instruction: Option<String>,
    /// Distance of this step in meters
    pub distance: Option<f64>,
    /// Duration of this step in seconds
    pub duration: Option<f64>,
    /// Starting coordinate of this step
    pub start_coordinate: Option<Coordinate>,
    /// Ending coordinate of this step
    pub end_coordinate: Option<Coordinate>,
}

/// Transport mode for a route.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransportMode {
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Scooter,
    Bus,
    Taxi,
    Unknown,
}

/// A unified route result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    /// Total route distance in meters
    pub distance: f64,
    /// Total route duration in seconds
    pub duration: f64,
    /// Route geometry as a polyline
    pub geometry: Polyline,
    /// Transport mode used for this route
    pub transport_mode: Option<TransportMode>,
    /// Turn-by-turn steps (if available)
    pub steps: Vec<RouteStep>,
    /// Bounding box for the route (if available)
    pub bounding_box: Option<BoundingBox>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// A unified route response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    /// The route results (may contain alternatives)
    pub routes: Vec<RouteResult>,
}

#[async_trait]
pub trait Router: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn calculate_route(&self, req: RouteRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}