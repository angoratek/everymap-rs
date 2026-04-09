use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for GPS trace matching to the road network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRequest<O> {
    pub points: Vec<Coordinate>,
    pub options: O,
}

/// Simplified matching response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResponse {
    pub snapped_points: Vec<Coordinate>,
    pub distance: f64,
}

#[async_trait]
pub trait RouteMatcher: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn match_route(&self, req: TraceRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}