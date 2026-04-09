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

/// A unified isoline result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineResult {
    /// The polygon points defining the isoline boundary
    pub polygon: Vec<Coordinate>,
    /// The range value (in meters or seconds, depending on range type)
    pub range: Option<f64>,
}

/// A unified isoline response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineResponse {
    /// The isoline results (may contain multiple ranges)
    pub isolines: Vec<IsolineResult>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait IsolineProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_isoline(&self, req: IsolineRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}