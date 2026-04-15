use everymap_core::types::Coordinate;
use everymap_core::domains::search::{SearchResult, SearchResultType, SearchResponse};
use crate::domain::geo::HereLatLng;
use serde::{Deserialize, Serialize};

// --- Shared enums used across multiple search endpoints ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    Address,
    Area,
    City,
    HouseNumber,
    Place,
    PostalCode,
    Street,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressNamesMode {
    #[default]
    Matched,
    Normalized,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostalCodeMode {
    #[default]
    CityLookup,
    DistrictLookup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WithFeature {
    Mpa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowFeature {
    AddressUsage,
    CountryInfo,
    Parsing,
    PostalCodeDetails,
    SecondaryUnitInfo,
    StreetInfo,
    Tz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowMapReference {
    AdminIds,
    CmVersion,
    Links,
    MicroPointAddress,
    PointAddress,
    Segments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowNavAttribute {
    Access,
    FunctionalClass,
    Physical,
    SpeedLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowRelated {
    Mpa,
    Intersections,
    ParentPA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowTranslation {
    City,
    County,
    District,
    State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverWithFeature {
    RecommendPlaces,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MobilityMode {
    Car,
    Motorbike,
    Truck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankingMode {
    ExcursionDistance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HereEvSupplyType {
    Ac,
    Dc,
}

// --- Rich response types ---

/// Full response from the HERE Geocoding & Search API v7.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereSearchResponse {
    #[serde(default)]
    pub items: Vec<HereSearchItem>,
}

impl From<HereSearchResponse> for SearchResponse {
    fn from(res: HereSearchResponse) -> Self {
        SearchResponse {
            items: res.items.into_iter().map(SearchResult::from).collect(),
        }
    }
}

/// A single search result item from the HERE Search API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereSearchItem {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "resultType")]
    pub result_type: Option<String>,
    #[serde(default)]
    pub address: Option<HereAddress>,
    #[serde(default)]
    pub position: Option<HereLatLng>,
    #[serde(default)]
    pub access: Vec<HereLatLng>,
    #[serde(default)]
    pub distance: Option<f64>,
    #[serde(default)]
    pub categories: Vec<HereCategory>,
    #[serde(default)]
    pub references: Vec<HereReference>,
    #[serde(default)]
    pub contacts: Vec<HereContact>,
    #[serde(default)]
    pub food: Option<HereFood>,
    #[serde(default, rename = "mapView")]
    pub map_view: Option<HereMapView>,
    #[serde(default, rename = "ontologyId")]
    pub ontology_id: Option<String>,
    #[serde(default)]
    pub chains: Vec<HereChain>,
    #[serde(default, rename = "addressBlock")]
    pub address_block: Option<HereAddressBlock>,
    #[serde(default)]
    pub highlights: Option<HereHighlights>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub geojson: Option<serde_json::Value>,
}

impl From<HereSearchItem> for SearchResult {
    fn from(item: HereSearchItem) -> Self {
        let coordinate = item.position
            .map(Coordinate::from)
            .unwrap_or(Coordinate::ORIGIN);
        let address = item.address
            .map(everymap_core::types::Address::from)
            .unwrap_or_default();
        let result_type = match item.result_type.as_deref() {
            Some("place") | Some("exactMatch") => SearchResultType::ExactMatch,
            Some("approximate") => SearchResultType::Approximate,
            Some("interpolated") => SearchResultType::Interpolated,
            _ => SearchResultType::Unknown,
        };
        let bounding_box = item.map_view.and_then(|mv| {
            match (mv.west, mv.south, mv.east, mv.north) {
                (Some(w), Some(s), Some(e), Some(n)) => {
                    everymap_core::types::BoundingBox::new(
                        Coordinate::new(n, e).ok()?,
                        Coordinate::new(s, w).ok()?,
                    ).into()
                }
                _ => None,
            }
        });
        SearchResult {
            id: item.id,
            coordinate,
            address,
            title: item.title,
            result_type,
            distance: item.distance,
            confidence: None,
            categories: item.categories.into_iter().filter_map(|c| c.name).collect(),
            bounding_box,
            raw: None,
        }
    }
}

/// Full address from the HERE Search API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAddress {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default, rename = "countryCode")]
    pub country_code: Option<String>,
    #[serde(default, rename = "countryName")]
    pub country_name: Option<String>,
    #[serde(default, rename = "stateCode")]
    pub state_code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default, rename = "countyCode")]
    pub county_code: Option<String>,
    #[serde(default)]
    pub county: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub district: Option<String>,
    #[serde(default)]
    pub street: Option<String>,
    #[serde(default, rename = "houseNumber")]
    pub house_number: Option<String>,
    #[serde(default, rename = "postalCode")]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub building: Option<String>,
    #[serde(default, rename = "subDistrict")]
    pub sub_district: Option<String>,
    #[serde(default)]
    pub block: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
}

impl From<HereAddress> for everymap_core::types::Address {
    fn from(addr: HereAddress) -> Self {
        everymap_core::types::Address {
            label: addr.label,
            street: addr.street,
            house_number: addr.house_number,
            city: addr.city,
            district: addr.district,
            sub_district: addr.sub_district,
            state: addr.state,
            state_code: addr.state_code,
            postal_code: addr.postal_code,
            country: addr.country_name,
            country_code: addr.country_code,
            county: addr.county,
            building: addr.building,
            block: addr.block,
            unit: addr.unit,
        }
    }
}

/// A category for a search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereCategory {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub primary: Option<bool>,
}

/// A reference to an external system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereReference {
    #[serde(default, rename = "type")]
    pub ref_type: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
}

/// Contact information for a place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereContact {
    #[serde(default)]
    pub phone: Vec<HerePhone>,
    #[serde(default)]
    pub fax: Vec<HereFax>,
    #[serde(default)]
    pub www: Vec<HereWebLink>,
    #[serde(default)]
    pub email: Vec<HereEmail>,
}

/// Phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerePhone {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Fax number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereFax {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Web link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereWebLink {
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereEmail {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Food/restaurant information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereFood {
    #[serde(default)]
    pub cuisines: Vec<String>,
    #[serde(default, rename = "mealTypes")]
    pub meal_types: Vec<String>,
    #[serde(default)]
    pub ratings: Vec<HereRating>,
}

/// Rating information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereRating {
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default)]
    pub provider: Option<String>,
}

/// Bounding box view for a map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereMapView {
    #[serde(default, rename = "west")]
    pub west: Option<f64>,
    #[serde(default, rename = "south")]
    pub south: Option<f64>,
    #[serde(default, rename = "east")]
    pub east: Option<f64>,
    #[serde(default, rename = "north")]
    pub north: Option<f64>,
}

/// Chain information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereChain {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
}

/// Address block information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAddressBlock {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub address: Option<HereAddress>,
}

/// Highlight information for search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereHighlights {
    #[serde(default)]
    pub title: Option<Vec<HereHighlightSection>>,
    #[serde(default)]
    pub address: Option<Vec<HereHighlightSection>>,
}

/// A section of highlighted text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereHighlightSection {
    #[serde(default)]
    pub start: Option<u32>,
    #[serde(default)]
    pub end: Option<u32>,
}

/// Full response from the HERE Discover API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereDiscoverResponse {
    #[serde(default)]
    pub items: Vec<HereSearchItem>,
}

/// Full response from the HERE Autosuggest API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAutosuggestResponse {
    #[serde(default)]
    pub items: Vec<HereAutosuggestItem>,
    #[serde(default, rename = "queryTerms")]
    pub query_terms: Vec<HereQueryTerm>,
}

/// A single autosuggest result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAutosuggestItem {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "resultType")]
    pub result_type: Option<String>,
    #[serde(default)]
    pub address: Option<HereAddress>,
    #[serde(default)]
    pub position: Option<HereLatLng>,
    #[serde(default)]
    pub distance: Option<f64>,
    #[serde(default)]
    pub categories: Vec<HereCategory>,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub highlights: Option<HereHighlights>,
}

/// A query term from autosuggest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereQueryTerm {
    #[serde(default)]
    pub term: Option<String>,
    #[serde(default, rename = "displayText")]
    pub display_text: Option<String>,
}

// --- Request option types ---

/// Fuel station filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereFuelStationFilter {
    #[serde(default)]
    pub current: Option<Vec<String>>,
}

/// EV station filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereEvStationFilter {
    #[serde(default)]
    pub current: Option<Vec<HereEvSupplyType>>,
}

/// Route filter for search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRouteFilter {
    #[serde(default)]
    pub waypoints: Option<String>,
}

/// Options for the HERE Discover endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereDiscoverOptions {
    pub at: Option<Coordinate>,
    pub in_filter: Option<String>,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub political_view: Option<String>,
    pub types: Option<Vec<SearchType>>,
    pub with: Option<Vec<DiscoverWithFeature>>,
    pub show: Option<Vec<ShowFeature>>,
    pub fuel_station: Option<HereFuelStationFilter>,
    pub ev_station: Option<HereEvStationFilter>,
    pub mobility_mode: Option<MobilityMode>,
    pub ranking: Option<RankingMode>,
    pub route: Option<HereRouteFilter>,
    pub offset: Option<u32>,
}

/// Options for the HERE Autosuggest endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereAutosuggestOptions {
    pub at: Option<Coordinate>,
    pub in_filter: Option<String>,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub political_view: Option<String>,
    pub types: Option<Vec<SearchType>>,
    pub with: Option<Vec<DiscoverWithFeature>>,
    pub show: Option<Vec<ShowFeature>>,
    pub show_map_references: Option<Vec<ShowMapReference>>,
    pub fuel_station: Option<HereFuelStationFilter>,
    pub ev_station: Option<HereEvStationFilter>,
    pub mobility_mode: Option<MobilityMode>,
    pub ranking: Option<RankingMode>,
    pub route: Option<HereRouteFilter>,
    pub terms_limit: Option<u32>,
    pub offset: Option<u32>,
}