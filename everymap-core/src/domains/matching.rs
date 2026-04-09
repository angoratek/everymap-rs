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

/// A matched point from route matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedPoint {
    /// The matched/snapped coordinate on the road network
    pub coordinate: Coordinate,
    /// Confidence score for this match (0.0-1.0)
    pub confidence: Option<f64>,
    /// Matched road name (if available)
    pub road_name: Option<String>,
}

/// A unified matching response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResponse {
    /// The snapped/matched points
    pub matched_points: Vec<MatchedPoint>,
    /// Total matched route distance in meters
    pub distance: f64,
    /// Total matched route duration in seconds
    pub duration: Option<f64>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait RouteMatcher: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn match_route(&self, req: TraceRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}