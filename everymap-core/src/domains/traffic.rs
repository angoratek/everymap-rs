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

/// Severity of a traffic incident.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Low,
    Minor,
    Major,
    Critical,
    Unknown,
}

/// A traffic incident (accident, construction, road closure, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficIncident {
    /// Unique identifier for this incident
    pub id: Option<String>,
    /// Type of incident (e.g., "accident", "construction", "roadClosure")
    pub incident_type: Option<String>,
    /// Severity level
    pub severity: Option<IncidentSeverity>,
    /// Human-readable description
    pub description: Option<String>,
    /// Road name or location description
    pub road_name: Option<String>,
    /// Start time (ISO 8601 string)
    pub start_time: Option<String>,
    /// End time (ISO 8601 string)
    pub end_time: Option<String>,
}

/// A traffic flow measurement for a road segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficFlow {
    /// Current speed in km/h or mph (depends on provider region)
    pub speed: Option<f64>,
    /// Free-flow speed (speed without traffic)
    pub free_flow_speed: Option<f64>,
    /// Jam factor (0.0 = no traffic, 10.0 = gridlock)
    pub jam_factor: Option<f64>,
    /// Confidence in the measurement (0.0-1.0)
    pub confidence: Option<f64>,
    /// Road name (if available)
    pub road_name: Option<String>,
}

/// A unified traffic response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficResponse {
    /// Traffic flow data for the requested area
    pub flows: Vec<TrafficFlow>,
    /// Active traffic incidents in the area
    pub incidents: Vec<TrafficIncident>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait TrafficProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_traffic(&self, req: TrafficRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}