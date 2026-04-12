use serde::{Deserialize, Serialize};

/// Response from TomTom Traffic Flow API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomFlowResponse {
    #[serde(default)]
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
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
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