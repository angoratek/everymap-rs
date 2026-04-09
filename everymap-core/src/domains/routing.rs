use async_trait::async_trait;
use crate::types::{Coordinate, Polyline};
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for route calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest<O> {
    pub start: Coordinate,
    pub end: Coordinate,
    pub options: O,
}

/// Simplified route response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub distance: f64,
    pub duration: f64,
    pub geometry: Polyline,
}

#[async_trait]
pub trait Router: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn calculate_route(&self, req: RouteRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}