use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
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
    async fn get_traffic(
        &self,
        location: &Coordinate,
        options: &TrafficOptions,
    ) -> EveryMapResult<TrafficResponse>;
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

    // --- IncidentSeverity all 5 variants serde roundtrip ---

    #[test]
    fn test_incident_severity_all_variants_serde() {
        let variants = [
            IncidentSeverity::Low,
            IncidentSeverity::Minor,
            IncidentSeverity::Major,
            IncidentSeverity::Critical,
            IncidentSeverity::Unknown,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: IncidentSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_incident_severity_all_variants_distinct() {
        let variants = [
            IncidentSeverity::Low,
            IncidentSeverity::Minor,
            IncidentSeverity::Major,
            IncidentSeverity::Critical,
            IncidentSeverity::Unknown,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- TrafficResponse serde roundtrip ---

    #[test]
    fn test_traffic_response_serde_roundtrip() {
        let response = TrafficResponse {
            flows: vec![TrafficFlow {
                speed: Some(80.0),
                free_flow_speed: Some(120.0),
                jam_factor: Some(2.5),
                confidence: Some(0.9),
                road_name: Some("A9".to_string()),
            }],
            incidents: vec![TrafficIncident {
                id: Some("inc-1".to_string()),
                incident_type: Some("accident".to_string()),
                severity: Some(IncidentSeverity::Major),
                description: Some("Multi-vehicle accident".to_string()),
                road_name: Some("A9".to_string()),
                start_time: Some("2024-01-01T08:00:00".to_string()),
                end_time: Some("2024-01-01T12:00:00".to_string()),
            }],
            raw: Some(serde_json::json!({"source": "here"})),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TrafficResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.flows.len(), 1);
        assert_eq!(back.incidents.len(), 1);
        assert_eq!(back.incidents[0].severity, Some(IncidentSeverity::Major));
        assert!(back.raw.is_some());
    }

    #[test]
    fn test_traffic_response_empty() {
        let response = TrafficResponse {
            flows: vec![],
            incidents: vec![],
            raw: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TrafficResponse = serde_json::from_str(&json).unwrap();
        assert!(back.flows.is_empty());
        assert!(back.incidents.is_empty());
    }

    // --- TrafficOptions serde roundtrip ---

    #[test]
    fn test_traffic_options_serde_roundtrip() {
        let opts = TrafficOptions {
            radius: Some(5000.0),
            language: Some("de".to_string()),
            include_incidents: Some(true),
            provider_extra: Some(serde_json::json!({"min_jam_factor": 4.0})),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: TrafficOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.radius, Some(5000.0));
        assert_eq!(back.include_incidents, Some(true));
        assert!(back.provider_extra.is_some());
    }

    // --- TrafficIncident serde roundtrip ---

    #[test]
    fn test_traffic_incident_serde_roundtrip() {
        let incident = TrafficIncident {
            id: Some("inc-42".to_string()),
            incident_type: Some("construction".to_string()),
            severity: Some(IncidentSeverity::Minor),
            description: Some("Road work, lane closed".to_string()),
            road_name: Some("B96".to_string()),
            start_time: Some("2024-03-01T06:00:00".to_string()),
            end_time: Some("2024-09-01T18:00:00".to_string()),
        };
        let json = serde_json::to_string(&incident).unwrap();
        let back: TrafficIncident = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, incident.id);
        assert_eq!(back.severity, incident.severity);
        assert_eq!(back.description, incident.description);
    }

    // --- TrafficFlow serde roundtrip ---

    #[test]
    fn test_traffic_flow_serde_roundtrip() {
        let flow = TrafficFlow {
            speed: Some(0.0),
            free_flow_speed: Some(100.0),
            jam_factor: Some(10.0),
            confidence: Some(0.5),
            road_name: Some("A100".to_string()),
        };
        let json = serde_json::to_string(&flow).unwrap();
        let back: TrafficFlow = serde_json::from_str(&json).unwrap();
        assert_eq!(back.speed, Some(0.0));
        assert_eq!(back.jam_factor, Some(10.0));
    }

    // --- Edge cases ---

    #[test]
    fn test_traffic_flow_gridlock() {
        let flow = TrafficFlow {
            speed: Some(0.0),
            free_flow_speed: Some(100.0),
            jam_factor: Some(10.0),
            confidence: Some(1.0),
            road_name: None,
        };
        assert_eq!(flow.speed, Some(0.0));
        assert_eq!(flow.jam_factor, Some(10.0));
    }

    #[test]
    fn test_traffic_incident_all_none_optional_fields() {
        let incident = TrafficIncident {
            id: None,
            incident_type: None,
            severity: None,
            description: None,
            road_name: None,
            start_time: None,
            end_time: None,
        };
        assert!(incident.id.is_none());
        assert!(incident.severity.is_none());
    }
}
