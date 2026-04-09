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

/// Simplified tour response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourResponse {
    pub optimized_stops: Vec<Coordinate>,
}

#[async_trait]
pub trait TourPlanner: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn optimize_tour(&self, req: TourRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}