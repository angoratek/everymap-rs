use serde::{Deserialize, Serialize};
use crate::domain::geo::{GoogleLatLng, GoogleBounds};

// ============================================================================
// Response types — Google Directions API
// ============================================================================

/// Full response from the Google Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDirectionsResponse {
    #[serde(default)]
    pub routes: Vec<GoogleRoute>,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(default, rename = "geocodedWaypoints")]
    pub geocoded_waypoints: Vec<GoogleGeocodedWaypoint>,
    #[serde(default, rename = "availableTravelModes")]
    pub available_travel_modes: Vec<String>,
}

/// A route from the Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoute {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub legs: Vec<GoogleRouteLeg>,
    #[serde(default, rename = "overviewPolyline")]
    pub overview_polyline: Option<GooglePolyline>,
    #[serde(default)]
    pub bounds: Option<GoogleBounds>,
    #[serde(default)]
    pub copyrights: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, rename = "waypointOrder")]
    pub waypoint_order: Vec<u32>,
}

/// A leg of a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRouteLeg {
    #[serde(default)]
    pub distance: Option<GoogleDistance>,
    #[serde(default)]
    pub duration: Option<GoogleDuration>,
    #[serde(default, rename = "durationInTraffic")]
    pub duration_in_traffic: Option<GoogleDuration>,
    #[serde(default, rename = "startLocation")]
    pub start_location: Option<GoogleLatLng>,
    #[serde(default, rename = "endLocation")]
    pub end_location: Option<GoogleLatLng>,
    #[serde(default, rename = "startAddress")]
    pub start_address: Option<String>,
    #[serde(default, rename = "endAddress")]
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
    #[serde(default, rename = "startLocation")]
    pub start_location: Option<GoogleLatLng>,
    #[serde(default, rename = "endLocation")]
    pub end_location: Option<GoogleLatLng>,
    #[serde(default, rename = "htmlInstructions")]
    pub html_instructions: Option<String>,
    #[serde(default)]
    pub maneuver: Option<String>,
    #[serde(default)]
    pub polyline: Option<GooglePolyline>,
    #[serde(default, rename = "travelMode")]
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
    #[serde(default, rename = "geocoderStatus")]
    pub geocoder_status: Option<String>,
    #[serde(default, rename = "placeId")]
    pub place_id: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
}