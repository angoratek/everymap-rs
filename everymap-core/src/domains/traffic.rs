use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for traffic data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRequest<O> {
    pub location: Coordinate,
    pub options: O,
}

/// Simplified traffic response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficResponse {
    pub jam_factor: f64,
    pub incidents: Vec<String>,
}

#[async_trait]
pub trait TrafficProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_traffic(&self, req: TrafficRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}