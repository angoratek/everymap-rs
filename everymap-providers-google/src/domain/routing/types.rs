use serde::{Deserialize, Serialize};

// ============================================================================
// Request types — Google Routes API v2 (computeRoutes)
// ============================================================================
// NOTE: Routes API v2 accepts camelCase JSON fields. All serde renames use
// the actual API field names.

/// Full request body for the Routes API v2 `computeRoutes` method.
#[derive(Debug, Clone, Serialize)]
pub struct GoogleRoutesRequest {
    pub origin: GoogleWaypoint,
    pub destination: GoogleWaypoint,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub intermediates: Vec<GoogleWaypoint>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "travelMode")]
    pub travel_mode: Option<GoogleTravelMode>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "computeAlternativeRoutes"
    )]
    pub compute_alternative_routes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "routeModifiers")]
    pub route_modifiers: Option<GoogleRouteModifiers>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "languageCode")]
    pub language_code: Option<String>,
    /// RFC 3339 timestamp (e.g. "2024-06-01T08:00:00Z").
    #[serde(skip_serializing_if = "Option::is_none", rename = "departureTime")]
    pub departure_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "optimizeWaypoints")]
    pub optimize_waypoints: Option<bool>,
    /// "METRIC" or "IMPERIAL" (unit system for localized response values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

/// A waypoint (origin, destination, or intermediate stop).
#[derive(Debug, Clone, Serialize)]
pub struct GoogleWaypoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<GoogleRoutesLocation>,
}

/// A location container holding a lat/lng pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoutesLocation {
    #[serde(rename = "latLng")]
    pub lat_lng: GoogleRoutesLatLng,
}

/// A latitude/longitude pair (Routes API v2 field names).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoutesLatLng {
    pub latitude: f64,
    pub longitude: f64,
}

/// Travel mode for Routes API v2 (`RouteTravelMode` enum values).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GoogleTravelMode {
    Drive,
    Walk,
    Bicycle,
    TwoWheeler,
    Transit,
}

/// Route restrictions for Routes API v2 (`RouteModifiers` message).
///
/// Note: Routes API v2 has no "avoid dirt roads" modifier, and
/// `avoidTunnels` requires `routingPreference: TRAFFIC_AWARE_OPTIMAL`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GoogleRouteModifiers {
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "avoidTolls")]
    pub avoid_tolls: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "avoidHighways")]
    pub avoid_highways: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "avoidFerries")]
    pub avoid_ferries: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not", rename = "avoidIndoor")]
    pub avoid_indoor: bool,
}

// ============================================================================
// Response types — Google Routes API v2 (computeRoutes)
// ============================================================================

/// Full response from the Routes API v2 `computeRoutes` method.
///
/// Errors are reported via HTTP status codes (4xx/5xx), not an in-body
/// status field like the legacy Directions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoutesResponse {
    #[serde(default)]
    pub routes: Vec<GoogleRoute>,
}

/// A route from the Routes API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoute {
    #[serde(default, rename = "distanceMeters")]
    pub distance_meters: Option<u64>,
    /// Duration as a seconds-suffixed string (e.g. "1234s").
    #[serde(default)]
    pub duration: Option<String>,
    /// Duration without traffic delays (seconds-suffixed string).
    #[serde(default, rename = "staticDuration")]
    pub static_duration: Option<String>,
    #[serde(default)]
    pub polyline: Option<GoogleRoutesPolyline>,
    #[serde(default)]
    pub legs: Vec<GoogleRouteLeg>,
    #[serde(default)]
    pub viewport: Option<GoogleViewport>,
    #[serde(default, rename = "travelMode")]
    pub travel_mode: Option<GoogleTravelMode>,
    #[serde(default, rename = "routeLabels")]
    pub route_labels: Vec<String>,
}

/// A leg of a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRouteLeg {
    #[serde(default, rename = "distanceMeters")]
    pub distance_meters: Option<u64>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default, rename = "startLocation")]
    pub start_location: Option<GoogleRoutesLocation>,
    #[serde(default, rename = "endLocation")]
    pub end_location: Option<GoogleRoutesLocation>,
    #[serde(default)]
    pub steps: Vec<GoogleRouteStep>,
}

/// A step within a route leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRouteStep {
    #[serde(default, rename = "distanceMeters")]
    pub distance_meters: Option<u64>,
    #[serde(default, rename = "staticDuration")]
    pub static_duration: Option<String>,
    #[serde(default)]
    pub polyline: Option<GoogleRoutesPolyline>,
    #[serde(default, rename = "startLocation")]
    pub start_location: Option<GoogleRoutesLocation>,
    #[serde(default, rename = "endLocation")]
    pub end_location: Option<GoogleRoutesLocation>,
    #[serde(default, rename = "navigationInstruction")]
    pub navigation_instruction: Option<GoogleNavigationInstruction>,
    #[serde(default, rename = "travelMode")]
    pub travel_mode: Option<GoogleTravelMode>,
}

/// Plain-text navigation guidance for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleNavigationInstruction {
    #[serde(default)]
    pub instructions: Option<String>,
}

/// Encoded polyline container (Routes API v2 shape).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleRoutesPolyline {
    #[serde(default, rename = "encodedPolyline")]
    pub encoded_polyline: Option<String>,
}

/// Viewport (high = northeast, low = southwest corner).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleViewport {
    #[serde(default)]
    pub high: Option<GoogleRoutesLatLng>,
    #[serde(default)]
    pub low: Option<GoogleRoutesLatLng>,
}
