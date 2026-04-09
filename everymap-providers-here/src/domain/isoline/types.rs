use serde::{Deserialize, Serialize};

/// Full response from the HERE Isoline Routing API v8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolineApiResponse {
    #[serde(default)]
    pub isolines: Vec<HereIsoline>,
    #[serde(default)]
    pub departures: Vec<HereDeparture>,
}

/// A single isoline (reachability polygon).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsoline {
    #[serde(default)]
    pub range: HereIsolineRange,
    #[serde(default)]
    pub polygons: Vec<HerePolygon>,
    #[serde(default)]
    pub connections: Vec<HereIsolineConnection>,
}

/// Range for an isoline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereIsolineRange {
    #[serde(default, rename = "type")]
    pub range_type: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
}

/// A polygon within an isoline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerePolygon {
    #[serde(default)]
    pub outer: Option<String>,
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
    pub location: Option<HereIsolineLatLng>,
    #[serde(default, rename = "originalLocation")]
    pub original_location: Option<HereIsolineLatLng>,
    #[serde(default, rename = "sideOfStreet")]
    pub side_of_street: Option<String>,
}

/// Lat/lng for isoline responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereIsolineLatLng {
    pub lat: f64,
    pub lng: f64,
}

impl From<HereIsolineLatLng> for everymap_core::types::Coordinate {
    fn from(val: HereIsolineLatLng) -> Self {
        everymap_core::types::Coordinate::new(val.lat, val.lng)
            .unwrap_or_else(|_| everymap_core::types::Coordinate::new(0.0, 0.0).unwrap())
    }
}