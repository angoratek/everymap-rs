use serde::{Deserialize, Serialize};

/// Full response from the HERE Route Matching API v8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereMatchApiResponse {
    #[serde(default)]
    pub trace: Vec<HereMatchedPoint>,
    #[serde(default)]
    pub route: Option<HereMatchedRoute>,
    #[serde(default)]
    pub summary: Option<HereMatchSummary>,
    #[serde(default)]
    pub errors: Vec<HereMatchError>,
}

/// A matched point from the route matching response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchedPoint {
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lng: Option<f64>,
    #[serde(default, rename = "linkId")]
    pub link_id: Option<String>,
    #[serde(default, rename = "linkMatchProbability")]
    pub link_match_probability: Option<f64>,
    #[serde(default, rename = "pointMatchProbability")]
    pub point_match_probability: Option<f64>,
    #[serde(default)]
    pub heading: Option<f64>,
    #[serde(default)]
    pub speed: Option<f64>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default, rename = "distanceFromStart")]
    pub distance_from_start: Option<f64>,
    #[serde(default, rename = "elevation")]
    pub elevation: Option<f64>,
    #[serde(default, rename = "matchQuality")]
    pub match_quality: Option<String>,
}

/// The matched route from the route matching response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchedRoute {
    #[serde(default, rename = "routeId")]
    pub route_id: Option<String>,
    #[serde(default)]
    pub legs: Vec<HereMatchedLeg>,
    #[serde(default)]
    pub summary: Option<HereMatchSummary>,
}

/// A leg of a matched route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchedLeg {
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "travelTime")]
    pub travel_time: Option<f64>,
    #[serde(default)]
    pub maneuvers: Vec<HereMatchedManeuver>,
    #[serde(default)]
    pub links: Vec<HereMatchedLink>,
}

/// A maneuver in a matched route leg.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchedManeuver {
    #[serde(default)]
    pub position: Option<HereMatchLatLng>,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub length: Option<f64>,
}

/// A link in a matched route leg.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchedLink {
    #[serde(default, rename = "linkId")]
    pub link_id: Option<String>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default)]
    pub shape: Option<String>,
    #[serde(default, rename = "functionalClass")]
    pub functional_class: Option<u32>,
    #[serde(default, rename = "speedLimit")]
    pub speed_limit: Option<f64>,
}

/// Lat/lng for matching responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchLatLng {
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lng: f64,
}

/// Summary of a matched route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchSummary {
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default, rename = "travelTime")]
    pub travel_time: Option<f64>,
    #[serde(default)]
    pub deviation: Option<f64>,
}

/// An error from route matching.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchError {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub index: Option<u32>,
}

// --- Parameter enums ---

/// Route match mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Bus,
    CarHov,
    Emergency,
    Motorcycle,
    RoadTrain,
    Taxi,
}

/// Legal constraint type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalConstraint {
    Strict,
    Relaxed,
}

/// Trailer type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrailerType {
    #[serde(rename = "0")]
    None,
    #[serde(rename = "1")]
    SingleAxle,
    #[serde(rename = "2")]
    DoubleAxle,
    #[serde(rename = "3")]
    SemiTrailer,
    #[serde(rename = "4")]
    BDouble,
}

/// Emission type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmissionType {
    #[serde(rename = "1")]
    Euro1,
    #[serde(rename = "2")]
    Euro2,
    #[serde(rename = "3")]
    Euro3,
    #[serde(rename = "4")]
    Euro4,
    #[serde(rename = "5")]
    Euro5,
    #[serde(rename = "6")]
    Euro6,
    #[serde(rename = "7")]
    EuroEev,
    #[serde(rename = "8")]
    EuroVi,
    #[serde(rename = "9")]
    EuroViA,
    #[serde(rename = "10")]
    EuroViB,
    #[serde(rename = "11")]
    EuroViC,
}

/// Fuel type for route matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchFuelType {
    Petrol,
    Diesel,
    Lpg,
    Cng,
    Ethanol,
    Propane,
    Hydrogen,
    Electric,
    PetrolElecHybrid,
    DieselElecHybrid,
}

/// Tunnel category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelCategory {
    B,
    C,
    D,
    E,
}

/// Hazardous goods type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardousGoodsType {
    Explosive,
    Gas,
    Flammable,
    Poison,
    RadioActive,
    Corrosive,
    HarmfulToWater,
    Other,
}

/// Instruction format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionFormat {
    Text,
    Html,
}

/// Avoid features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvoidFeature {
    TollRoad,
    Ferry,
    CarShuttleTrain,
    Tunnel,
    DirtRoad,
    SeasonalClosure,
    ControlledAccessHighway,
}