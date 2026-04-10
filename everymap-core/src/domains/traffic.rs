use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for traffic data retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficOptions {
    /// Search radius in meters from the location
    pub radius: Option<f64>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Whether to include traffic incidents in the response
    pub include_incidents: Option<bool>,
    /// Provider-specific options (HERE: min_jam_factor, functional_classes; TomTom: thickness)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
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
    async fn get_traffic(&self, location: &Coordinate, options: &TrafficOptions) -> EveryMapResult<TrafficResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_options_default() {
        let opts = TrafficOptions::default();
        assert!(opts.radius.is_none());
        assert!(opts.language.is_none());
        assert!(opts.include_incidents.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_incident_severity_serialization() {
        assert_eq!(
            serde_json::to_string(&IncidentSeverity::Critical).unwrap(),
            "\"Critical\""
        );
        let sev: IncidentSeverity = serde_json::from_str("\"Minor\"").unwrap();
        assert_eq!(sev, IncidentSeverity::Minor);
    }

    #[test]
    fn test_traffic_flow_construction() {
        let flow = TrafficFlow {
            speed: Some(80.0),
            free_flow_speed: Some(100.0),
            jam_factor: Some(3.5),
            confidence: Some(0.9),
            road_name: Some("A100".to_string()),
        };
        assert_eq!(flow.speed, Some(80.0));
        assert_eq!(flow.jam_factor, Some(3.5));
        assert_eq!(flow.road_name.as_deref(), Some("A100"));
    }

    #[test]
    fn test_traffic_response_construction() {
        let response = TrafficResponse {
            flows: vec![TrafficFlow {
                speed: Some(60.0),
                free_flow_speed: Some(100.0),
                jam_factor: Some(5.0),
                confidence: Some(0.85),
                road_name: None,
            }],
            incidents: vec![],
            raw: None,
        };
        assert_eq!(response.flows.len(), 1);
        assert!(response.incidents.is_empty());
    }
}