use serde::{Deserialize, Serialize};

/// Full response from the HERE Isoline Routing API v8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolineApiResponse {
    /// The reachability polygons the service computed. The service always
    /// reports them, so a payload without this field is not an isoline
    /// response.
    pub isolines: Vec<HereIsoline>,
    #[serde(default)]
    pub departures: Vec<HereDeparture>,
}

/// A single isoline (reachability polygon).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsoline {
    /// The range this polygon covers. Every isoline carries one.
    pub range: HereIsolineRange,
    #[serde(default)]
    pub polygons: Vec<HerePolygon>,
    #[serde(default)]
    pub connections: Vec<HereIsolineConnection>,
}

/// Range for an isoline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolineRange {
    #[serde(rename = "type")]
    pub range_type: String,
    pub value: f64,
}

/// A polygon within an isoline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerePolygon {
    /// The encoded outer ring. A polygon without a ring is not usable.
    pub outer: String,
    #[serde(default)]
    pub holes: Option<Vec<String>>,
}

/// A connection between isolines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolineConnection {
    #[serde(default, rename = "fromIndex")]
    pub from_index: Option<u32>,
    #[serde(default, rename = "toIndex")]
    pub to_index: Option<u32>,
    #[serde(default, rename = "fromRef")]
    pub from_ref: Option<String>,
    #[serde(default, rename = "toRef")]
    pub to_ref: Option<String>,
}

/// Departure information for an isoline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereDeparture {
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub place: Option<HereIsolinePlace>,
}

/// A place reference in an isoline response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolinePlace {
    #[serde(default, rename = "type")]
    pub place_type: Option<String>,
    #[serde(default)]
    pub location: Option<crate::domain::geo::HereLatLng>,
    #[serde(default, rename = "originalLocation")]
    pub original_location: Option<crate::domain::geo::HereLatLng>,
    #[serde(default, rename = "sideOfStreet")]
    pub side_of_street: Option<String>,
}

// HereIsolineLatLng has been consolidated into crate::domain::geo::HereLatLng
