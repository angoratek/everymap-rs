use serde::{Deserialize, Serialize};
use crate::domain::geo::HereLatLng;

/// Full response from the HERE Routing API v8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRouteApiResponse {
    #[serde(default)]
    pub routes: Vec<HereRoute>,
    #[serde(default, rename = "notices")]
    pub notices: Vec<HereNotice>,
}

/// A single route from the HERE Routing API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRoute {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "routeHandle")]
    pub route_handle: Option<String>,
    #[serde(default)]
    pub sections: Vec<HereRouteSection>,
}

/// A section of a route (one per transport mode leg).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRouteSection {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "type")]
    pub section_type: Option<String>,
    #[serde(default)]
    pub departure: Option<HerePlace>,
    #[serde(default)]
    pub arrival: Option<HerePlace>,
    #[serde(default)]
    pub summary: Option<HereRouteSummary>,
    #[serde(default)]
    pub polyline: Option<HerePolylineField>,
    #[serde(default)]
    pub actions: Vec<HereRouteAction>,
    #[serde(default, rename = "turnByTurnActions")]
    pub turn_by_turn_actions: Vec<HereTurnAction>,
    #[serde(default)]
    pub notices: Vec<HereNotice>,
    #[serde(default, rename = "travelSummary")]
    pub travel_summary: Option<HereTravelSummary>,
    #[serde(default)]
    pub spans: Vec<HereSpan>,
    #[serde(default, rename = "transportMode")]
    pub transport_mode: Option<String>,
}

/// Summary of a route section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRouteSummary {
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "baseDuration")]
    pub base_duration: Option<f64>,
    #[serde(default, rename = "typicalDuration")]
    pub typical_duration: Option<f64>,
}

/// Travel summary with additional details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereTravelSummary {
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "baseDuration")]
    pub base_duration: Option<f64>,
}

/// A place (departure/arrival) along a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerePlace {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub place_type: Option<String>,
    #[serde(default)]
    pub location: Option<HereLatLng>,
    #[serde(default, rename = "originalLocation")]
    pub original_location: Option<HereLatLng>,
    #[serde(default, rename = "sideOfStreet")]
    pub side_of_street: Option<String>,
    #[serde(default)]
    pub waypoint: Option<HereWaypointInfo>,
}

/// Waypoint information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereWaypointInfo {
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Polyline data in a route section.
/// The HERE Routing API v8 returns the polyline as a plain string,
/// not as an object: `"polyline": "BGoz5xJ67i1B1B7PzIhaxL7Y"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerePolylineData {
    #[serde(default)]
    pub polyline: Option<String>,
}

/// Wrapper that deserializes a polyline that may be either:
/// - a plain string: `"polyline": "BGoz5xJ67i1B1B7PzIhaxL7Y"`
/// - an object: `"polyline": { "polyline": "BGoz5xJ67i1B1B7PzIhaxL7Y" }`
///
/// The real HERE Routing API v8 returns a plain string, but the contract
/// tests use the object form for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HerePolylineField {
    /// The polyline is a plain flexible-polyline encoded string.
    String(String),
    /// The polyline is an object with a nested `polyline` field.
    Object(HerePolylineData),
}

impl HerePolylineField {
    /// Extract the encoded polyline string from either variant.
    pub fn into_polyline_string(self) -> Option<String> {
        match self {
            HerePolylineField::String(s) => Some(s),
            HerePolylineField::Object(obj) => obj.polyline,
        }
    }
}

/// A route action (maneuver).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRouteAction {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default, rename = "nextAction")]
    pub next_action: Option<HereNextAction>,
}

/// Next action reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereNextAction {
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub direction: Option<String>,
}

/// Turn-by-turn action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereTurnAction {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default, rename = "streetName")]
    pub street_name: Option<String>,
    #[serde(default, rename = "nextStreetName")]
    pub next_street_name: Option<String>,
}

/// A notice/warning from the routing API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereNotice {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
}

/// A span of route data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereSpan {
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default, rename = "countryCode")]
    pub country_code: Option<String>,
    #[serde(default, rename = "functionalClass")]
    pub functional_class: Option<u32>,
    #[serde(default, rename = "speedLimit")]
    pub speed_limit: Option<f64>,
    #[serde(default, rename = "streetAttributes")]
    pub street_attributes: Option<Vec<String>>,
}