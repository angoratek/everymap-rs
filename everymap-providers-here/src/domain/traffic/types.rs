use serde::{Deserialize, Serialize};

/// Full response from the HERE Traffic Flow API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereFlowResponse {
    #[serde(default)]
    pub results: Vec<HereFlowItem>,
}

/// A single traffic flow result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereFlowItem {
    #[serde(default)]
    pub location: HereTrafficLocation,
    #[serde(default, rename = "currentFlow")]
    pub current_flow: HereCurrentFlow,
    #[serde(default, rename = "roadInfo")]
    pub road_info: Option<HereRoadInfo>,
}

/// Current flow data for a road segment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereCurrentFlow {
    #[serde(default, rename = "speed")]
    pub speed: Option<f64>,
    #[serde(default, rename = "speedUncapped")]
    pub speed_uncapped: Option<f64>,
    #[serde(default, rename = "freeFlow")]
    pub free_flow: Option<f64>,
    #[serde(default, rename = "jamFactor")]
    pub jam_factor: Option<f64>,
    #[serde(default, rename = "jamTendency")]
    pub jam_tendency: Option<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub traversability: Option<String>,
    #[serde(default, rename = "subSegments")]
    pub sub_segments: Option<Vec<HereSubSegment>>,
    #[serde(default)]
    pub lanes: Option<Vec<HereLane>>,
}

/// Sub-segment of traffic flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereSubSegment {
    #[serde(default, rename = "jamFactor")]
    pub jam_factor: Option<f64>,
    #[serde(default)]
    pub speed: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
}

/// Lane-level traffic flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereLane {
    #[serde(default)]
    pub index: Option<u32>,
    #[serde(default, rename = "jamFactor")]
    pub jam_factor: Option<f64>,
    #[serde(default)]
    pub speed: Option<f64>,
}

/// Road information for a traffic segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRoadInfo {
    #[serde(default, rename = "functionalClass")]
    pub functional_class: Option<u32>,
    #[serde(default)]
    pub road_name: Option<String>,
    #[serde(default)]
    pub road_shield: Option<String>,
}

/// Traffic location reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereTrafficLocation {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "functionalClass")]
    pub functional_class: Option<u32>,
    #[serde(default)]
    pub shape: Option<Vec<HereTrafficShapePoint>>,
}

/// A shape point in a traffic location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereTrafficShapePoint {
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lng: f64,
}

/// Full response from the HERE Traffic Incidents API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIncidentsResponse {
    #[serde(default)]
    pub results: Vec<HereIncidentItem>,
}

/// A single traffic incident result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIncidentItem {
    #[serde(default)]
    pub location: HereTrafficLocation,
    #[serde(default)]
    pub incident: HereIncident,
}

/// A traffic incident.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereIncident {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "originalId")]
    pub original_id: Option<String>,
    #[serde(default, rename = "type")]
    pub incident_type: Option<String>,
    #[serde(default)]
    pub criticality: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(default, rename = "endTime")]
    pub end_time: Option<String>,
    #[serde(default)]
    pub description: Option<HereIncidentDescription>,
    #[serde(default)]
    pub summary: Option<Vec<HereIncidentDescription>>,
    #[serde(default)]
    pub items: Option<Vec<HereAffectedItem>>,
}

/// Incident description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIncidentDescription {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

/// An affected item within an incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAffectedItem {
    #[serde(default, rename = "type")]
    pub item_type: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

/// Options for the HERE Traffic Flow API.
#[derive(Debug, Clone, Default)]
pub struct HereFlowOptions {
    pub in_filter: Option<String>,
    pub location_referencing: Option<Vec<LocationReferencing>>,
    pub min_jam_factor: Option<f64>,
    pub max_jam_factor: Option<f64>,
    pub functional_classes: Option<Vec<u32>>,
    pub advanced_features: Option<Vec<AdvancedFeature>>,
    pub use_ref_replacements: Option<bool>,
    pub exact_segment_ref_matching: Option<bool>,
}

/// Options for the HERE Traffic Incidents API.
#[derive(Debug, Clone, Default)]
pub struct HereIncidentsOptions {
    pub in_filter: Option<String>,
    pub location_referencing: Option<Vec<LocationReferencing>>,
    pub functional_classes: Option<Vec<u32>>,
    pub criticality: Option<Vec<CriticalityLevel>>,
    pub incident_types: Option<Vec<IncidentType>>,
    pub earliest_start_time: Option<String>,
    pub latest_end_time: Option<String>,
    pub lang: Option<String>,
    pub units: Option<TrafficUnits>,
    pub use_ref_replacements: Option<bool>,
    pub exact_segment_ref_matching: Option<bool>,
}

/// Location referencing format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationReferencing {
    None,
    Olr,
    Shape,
    Tmc,
    #[serde(rename = "segmentRef")]
    SegmentRef,
}

/// Advanced traffic flow features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedFeature {
    DeepCoverage,
    Lanes,
}

/// Incident criticality level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriticalityLevel {
    Low,
    Minor,
    Major,
    Critical,
}

/// Traffic incident type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentType {
    Accident,
    Construction,
    Congestion,
    DisabledVehicle,
    MassTransit,
    PlannedEvent,
    RoadHazard,
    RoadClosure,
    Weather,
    LaneRestriction,
    Other,
}

/// Traffic units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficUnits {
    Metric,
    Imperial,
}

// --- From conversions to core types ---

impl From<HereFlowItem> for everymap_core::domains::traffic::TrafficFlow {
    fn from(item: HereFlowItem) -> Self {
        Self {
            speed: item.current_flow.speed,
            free_flow_speed: item.current_flow.free_flow,
            jam_factor: item.current_flow.jam_factor,
            confidence: item.current_flow.confidence,
            road_name: item.road_info.and_then(|ri| ri.road_name),
        }
    }
}

impl From<HereIncident> for everymap_core::domains::traffic::TrafficIncident {
    fn from(incident: HereIncident) -> Self {
        use everymap_core::domains::traffic::IncidentSeverity;
        let severity = incident.criticality.as_deref().map(|c| match c {
            "low" => IncidentSeverity::Low,
            "minor" => IncidentSeverity::Minor,
            "major" => IncidentSeverity::Major,
            "critical" => IncidentSeverity::Critical,
            _ => IncidentSeverity::Unknown,
        });
        let description = incident.description.and_then(|d| d.value);
        Self {
            id: incident.id,
            incident_type: incident.incident_type,
            severity,
            description,
            road_name: None, // Not directly available on HereIncident
            start_time: incident.start_time,
            end_time: incident.end_time,
        }
    }
}