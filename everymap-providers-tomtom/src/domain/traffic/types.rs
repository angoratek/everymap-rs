use everymap_core::domains::traffic::{IncidentSeverity, TrafficFlow, TrafficIncident};
use serde::{Deserialize, Serialize};

/// Calculate jam factor from current/free-flow speed ratio.
fn calculate_jam_factor(current: f64, free_flow: f64) -> f64 {
    if free_flow <= 0.0 {
        return 0.0;
    }
    let ratio = current / free_flow;
    if ratio >= 1.0 {
        0.0
    } else {
        (1.0 - ratio) * 10.0
    }
}

/// Map TomTom severity string to core IncidentSeverity.
fn map_severity(severity: &str) -> IncidentSeverity {
    match severity.to_lowercase().as_str() {
        "minor" => IncidentSeverity::Minor,
        "moderate" => IncidentSeverity::Moderate,
        "major" => IncidentSeverity::Major,
        "critical" => IncidentSeverity::Critical,
        _ => IncidentSeverity::Unknown,
    }
}

impl From<TomTomFlowSegmentData> for TrafficFlow {
    fn from(seg: TomTomFlowSegmentData) -> Self {
        let jam_factor = calculate_jam_factor(seg.current_speed, seg.free_flow_speed);
        TrafficFlow {
            speed: Some(seg.current_speed),
            free_flow_speed: Some(seg.free_flow_speed),
            jam_factor: Some(jam_factor),
            confidence: seg.confidence,
            road_name: seg.road_name,
        }
    }
}

impl From<TomTomIncident> for TrafficIncident {
    fn from(inc: TomTomIncident) -> Self {
        TrafficIncident {
            id: inc.id,
            incident_type: inc.incident_type,
            severity: inc.severity.as_deref().map(map_severity),
            description: inc.description,
            road_name: None,
            start_time: inc.start_time,
            end_time: inc.end_time,
        }
    }
}

/// Response from TomTom Traffic Flow API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomFlowResponse {
    #[serde(default, rename = "flowSegmentData")]
    pub flow_segment_data: Option<TomTomFlowSegmentData>,
}

/// Flow segment data from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomFlowSegmentData {
    #[serde(default, rename = "frc")]
    pub frc: Option<String>,
    #[serde(default, rename = "currentSpeed")]
    pub current_speed: f64,
    #[serde(default, rename = "freeFlowSpeed")]
    pub free_flow_speed: f64,
    #[serde(default, rename = "currentTravelTime")]
    pub current_travel_time: Option<u64>,
    #[serde(default, rename = "freeFlowTravelTime")]
    pub free_flow_travel_time: Option<u64>,
    #[serde(default, rename = "confidence")]
    pub confidence: Option<f64>,
    #[serde(default, rename = "roadName")]
    pub road_name: Option<String>,
    #[serde(default)]
    pub coordinates: Option<TomTomFlowCoordinates>,
}

/// Coordinates for a flow segment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomFlowCoordinates {
    #[serde(default)]
    pub coordinate: Vec<TomTomFlowCoordinate>,
}

/// A single coordinate in flow data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomFlowCoordinate {
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

/// Response from TomTom Traffic Incidents API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomIncidentsResponse {
    #[serde(default)]
    pub incidents: Vec<TomTomIncident>,
}

/// A traffic incident from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomIncident {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "type")]
    pub incident_type: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "from")]
    pub from_description: Option<String>,
    #[serde(default, rename = "to")]
    pub to_description: Option<String>,
    #[serde(default)]
    pub delay: Option<u64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(default, rename = "endTime")]
    pub end_time: Option<String>,
    #[serde(default)]
    pub geometry: Option<TomTomIncidentGeometry>,
}

/// Incident geometry from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomIncidentGeometry {
    #[serde(rename = "type", default)]
    pub geometry_type: Option<String>,
    #[serde(default)]
    pub coordinates: Option<serde_json::Value>,
}
