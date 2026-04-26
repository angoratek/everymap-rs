use crate::domain::geo::{GoogleBounds, GoogleLatLng};
use serde::{Deserialize, Serialize};

// ============================================================================
// Response types — Google Directions API
// ============================================================================
// NOTE: The Google Directions API returns snake_case JSON fields.
// All serde renames use the actual API field names.

/// Full response from the Google Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDirectionsResponse {
    #[serde(default)]
    pub routes: Vec<GoogleRoute>,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "error_message")]
    pub error_message: Option<String>,
    #[serde(default, rename = "geocoded_waypoints")]
    pub geocoded_waypoints: Vec<GoogleGeocodedWaypoint>,
    #[serde(default, rename = "available_travel_modes")]
    pub available_travel_modes: Vec<String>,
}

/// A route from the Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoute {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub legs: Vec<GoogleRouteLeg>,
    #[serde(default, rename = "overview_polyline")]
    pub overview_polyline: Option<GooglePolyline>,
    #[serde(default)]
    pub bounds: Option<GoogleBounds>,
    #[serde(default)]
    pub copyrights: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, rename = "waypoint_order")]
    pub waypoint_order: Vec<u32>,
}

/// A leg of a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRouteLeg {
    #[serde(default)]
    pub distance: Option<GoogleDistance>,
    #[serde(default)]
    pub duration: Option<GoogleDuration>,
    #[serde(default, rename = "duration_in_traffic")]
    pub duration_in_traffic: Option<GoogleDuration>,
    #[serde(default, rename = "start_location")]
    pub start_location: Option<GoogleLatLng>,
    #[serde(default, rename = "end_location")]
    pub end_location: Option<GoogleLatLng>,
    #[serde(default, rename = "start_address")]
    pub start_address: Option<String>,
    #[serde(default, rename = "end_address")]
    pub end_address: Option<String>,
    #[serde(default)]
    pub steps: Vec<GoogleRouteStep>,
}

/// A step within a route leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRouteStep {
    #[serde(default)]
    pub distance: Option<GoogleDistance>,
    #[serde(default)]
    pub duration: Option<GoogleDuration>,
    #[serde(default, rename = "start_location")]
    pub start_location: Option<GoogleLatLng>,
    #[serde(default, rename = "end_location")]
    pub end_location: Option<GoogleLatLng>,
    #[serde(default, rename = "html_instructions")]
    pub html_instructions: Option<String>,
    #[serde(default)]
    pub maneuver: Option<String>,
    #[serde(default)]
    pub polyline: Option<GooglePolyline>,
    #[serde(default, rename = "travel_mode")]
    pub travel_mode: Option<String>,
}

/// Distance container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDistance {
    pub value: u64,
    pub text: String,
}

/// Duration container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDuration {
    pub value: u64,
    pub text: String,
}

/// Encoded polyline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooglePolyline {
    pub points: String,
}

/// Geocoded waypoint information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeocodedWaypoint {
    #[serde(default, rename = "geocoder_status")]
    pub geocoder_status: Option<String>,
    #[serde(default, rename = "place_id")]
    pub place_id: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
}
