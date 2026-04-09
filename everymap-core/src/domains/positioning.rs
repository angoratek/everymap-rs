use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for network-based positioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositioningRequest<O> {
    pub options: O,
}

/// A unified positioning response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositioningResponse {
    /// The estimated position
    pub coordinate: Coordinate,
    /// Accuracy of the position estimate in meters
    pub accuracy: Option<f64>,
    /// Altitude in meters above sea level (if available)
    pub altitude: Option<f64>,
    /// Accuracy of the altitude estimate in meters
    pub altitude_accuracy: Option<f64>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait NetworkPositioner: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_position(&self, req: PositioningRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}