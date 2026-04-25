use serde::{Deserialize, Serialize};

/// Response from TomTom Routing API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomRouteResponse {
    /// List of routes.
    #[serde(default)]
    pub routes: Vec<TomTomRoute>,
    /// Format version.
    #[serde(default, rename = "formatVersion")]
    pub format_version: Option<String>,
}

/// A single route from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomRoute {
    /// Route summary.
    #[serde(default)]
    pub summary: Option<TomTomRouteSummary>,
    /// Route legs.
    #[serde(default)]
    pub legs: Vec<TomTomRouteLeg>,
}

/// Route summary from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomRouteSummary {
    /// Length in meters.
    #[serde(default, rename = "lengthInMeters")]
    pub length_in_meters: u64,
    /// Travel time in seconds.
    #[serde(default, rename = "travelTimeInSeconds")]
    pub travel_time_in_seconds: u64,
    /// Traffic delay in seconds.
    #[serde(default, rename = "trafficDelayInSeconds")]
    pub traffic_delay_in_seconds: Option<u64>,
    /// Traffic length in meters.
    #[serde(default, rename = "trafficLengthInMeters")]
    pub traffic_length_in_meters: Option<u64>,
    /// Departure time.
    #[serde(default, rename = "departureTime")]
    pub departure_time: Option<String>,
    /// Arrival time.
    #[serde(default, rename = "arrivalTime")]
    pub arrival_time: Option<String>,
}

/// A route leg from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomRouteLeg {
    /// Summary of the leg.
    #[serde(default)]
    pub summary: Option<TomTomRouteSummary>,
    /// Points along the leg.
    #[serde(default)]
    pub points: Vec<TomTomRoutePoint>,
}

/// A point on a route from TomTom.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomTomRoutePoint {
    /// Latitude.
    #[serde(default)]
    pub latitude: f64,
    /// Longitude.
    #[serde(default)]
    pub longitude: f64,
}
