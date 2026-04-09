use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for isoline (reachability polygon) calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineRequest<O> {
    pub center: Coordinate,
    pub range: f64,
    pub options: O,
}

/// Simplified isoline response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineResponse {
    pub polygon: Vec<Coordinate>,
}

#[async_trait]
pub trait IsolineProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_isoline(&self, req: IsolineRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}