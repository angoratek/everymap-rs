use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for network-based positioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositioningRequest<O> {
    pub options: O,
}

/// Simplified positioning response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositioningResponse {
    pub coordinate: Coordinate,
    pub accuracy: Option<f64>,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
}

#[async_trait]
pub trait NetworkPositioner: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_position(&self, req: PositioningRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}