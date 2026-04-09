pub mod types;

use async_trait::async_trait;
use everymap_core::domains::search::{
    Geocoder, GeocodeRequest, ReverseGeocodeRequest,
    DiscoverRequest, AutosuggestRequest,
    SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{Coordinate, Address};
use crate::client::HereClient;
use crate::domain::geo::HereLatLng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Re-export all public types from the types module
pub use types::*;

const GEOCODE_BASE_URL: &str = "https://geocode.search.hereapi.com/v1";
const DISCOVER_BASE_URL: &str = "https://discover.search.hereapi.com/v1";
const AUTOSUGGEST_BASE_URL: &str = "https://autosuggest.search.hereapi.com/v1";

/// Exhaustive options for HERE Geocoding & Search API v7 geocode/reverseGeocode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereGeocodeOptions {
    pub at: Option<Coordinate>,
    pub in_filter: Option<String>,
    pub qq: Option<String>,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub political_view: Option<String>,
    pub address_names_mode: Option<AddressNamesMode>,
    pub address_names_variant: Option<String>,
    pub postal_code_mode: Option<PostalCodeMode>,
    pub types: Option<Vec<SearchType>>,
    pub with: Option<Vec<WithFeature>>,
    pub show: Option<Vec<ShowFeature>>,
    pub show_map_references: Option<Vec<ShowMapReference>>,
    pub show_nav_attributes: Option<Vec<ShowNavAttribute>>,
    pub show_related: Option<Vec<ShowRelated>>,
    pub show_translations: Option<Vec<ShowTranslation>>,
    pub request_id: Option<String>,
}

/// Implementation of Geocoder for HERE Technologies.
///
/// In addition to the core `Geocoder` trait methods (geocode, reverse_geocode),
/// this struct provides HERE-specific methods: `discover` and `autosuggest`.
pub struct HereGeocoder {
    client: Arc<HereClient>,
    geocode_base_url: String,
    discover_base_url: String,
    autosuggest_base_url: String,
}

impl HereGeocoder {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            geocode_base_url: GEOCODE_BASE_URL.to_string(),
            discover_base_url: DISCOVER_BASE_URL.to_string(),
            autosuggest_base_url: AUTOSUGGEST_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self {
            client,
            geocode_base_url: base_url.clone(),
            discover_base_url: base_url.clone(),
            autosuggest_base_url: base_url,
        }
    }

    /// Discover places/POIs matching a query.
    pub async fn discover(
        &self,
        req: DiscoverRequest<HereDiscoverOptions>,
    ) -> EveryMapResult<HereDiscoverResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();
        params.push(("q", req.query));
        apply_discover_options(&req.options, &mut params);

        let url = format!("{}/discover", self.discover_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereDiscoverResponse = response.json().await?;

        Ok(here_res)
    }

    /// Get autosuggest results for a partial query (type-ahead).
    pub async fn autosuggest(
        &self,
        req: AutosuggestRequest<HereAutosuggestOptions>,
    ) -> EveryMapResult<HereAutosuggestResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();
        params.push(("q", req.query));
        apply_autosuggest_options(&req.options, &mut params);

        let url = format!("{}/autosuggest", self.autosuggest_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereAutosuggestResponse = response.json().await?;

        Ok(here_res)
    }
}

// --- Internal deserialization types for geocode/reverseGeocode ---

#[derive(Debug, Deserialize)]
struct HereGeocodeApiResponse {
    items: Vec<HereGeocodeItem>,
}

#[derive(Debug, Deserialize)]
struct HereGeocodeItem {
    #[serde(default)]
    position: Option<HereLatLng>,
    #[serde(default)]
    address: Option<HereAddress>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "resultType")]
    result_type: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    distance: Option<f64>,
}

/// Helper to apply geocode options to query params.
fn apply_geocode_options(opts: &HereGeocodeOptions, params: &mut Vec<(&str, String)>) {
    if let Some(at) = &opts.at {
        params.push(("at", format!("{},{}", at.lat, at.lng)));
    }
    if let Some(in_filter) = &opts.in_filter {
        params.push(("in", in_filter.clone()));
    }
    if let Some(qq) = &opts.qq {
        params.push(("qq", qq.clone()));
    }
    if let Some(lang) = &opts.lang {
        params.push(("lang", lang.clone()));
    }
    if let Some(limit) = opts.limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(pv) = &opts.political_view {
        params.push(("politicalView", pv.clone()));
    }
    if let Some(anm) = &opts.address_names_mode {
        let val = serde_json::to_value(anm).unwrap().as_str().unwrap().to_string();
        params.push(("addressNamesMode", val));
    }
    if let Some(anv) = &opts.address_names_variant {
        params.push(("addressNamesVariant", anv.clone()));
    }
    if let Some(pcm) = &opts.postal_code_mode {
        let val = serde_json::to_value(pcm).unwrap().as_str().unwrap().to_string();
        params.push(("postalCodeMode", val));
    }
    if let Some(types) = &opts.types {
        let val = types.iter()
            .map(|t| serde_json::to_value(t).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("types", val));
    }
    if let Some(with) = &opts.with {
        let val = with.iter()
            .map(|w| serde_json::to_value(w).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("with", val));
    }
    if let Some(show) = &opts.show {
        let val = show.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("show", val));
    }
    if let Some(smr) = &opts.show_map_references {
        let val = smr.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("showMapReferences", val));
    }
    if let Some(sna) = &opts.show_nav_attributes {
        let val = sna.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("showNavAttributes", val));
    }
    if let Some(sr) = &opts.show_related {
        let val = sr.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("showRelated", val));
    }
    if let Some(st) = &opts.show_translations {
        let val = st.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("showTranslations", val));
    }
    if let Some(rid) = &opts.request_id {
        params.push(("X-Request-ID", rid.clone()));
    }
}

/// Helper to apply discover options to query params.
fn apply_discover_options(opts: &HereDiscoverOptions, params: &mut Vec<(&str, String)>) {
    if let Some(at) = &opts.at {
        params.push(("at", format!("{},{}", at.lat, at.lng)));
    }
    if let Some(in_filter) = &opts.in_filter {
        params.push(("in", in_filter.clone()));
    }
    if let Some(lang) = &opts.lang {
        params.push(("lang", lang.clone()));
    }
    if let Some(limit) = opts.limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(pv) = &opts.political_view {
        params.push(("politicalView", pv.clone()));
    }
    if let Some(types) = &opts.types {
        let val = types.iter()
            .map(|t| serde_json::to_value(t).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("types", val));
    }
    if let Some(with) = &opts.with {
        let val = with.iter()
            .map(|w| serde_json::to_value(w).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("with", val));
    }
    if let Some(show) = &opts.show {
        let val = show.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("show", val));
    }
    if let Some(mobility) = &opts.mobility_mode {
        let val = serde_json::to_value(mobility).unwrap().as_str().unwrap().to_string();
        params.push(("mobilityMode", val));
    }
    if let Some(ranking) = &opts.ranking {
        let val = serde_json::to_value(ranking).unwrap().as_str().unwrap().to_string();
        params.push(("ranking", val));
    }
    if let Some(offset) = opts.offset {
        params.push(("offset", offset.to_string()));
    }
}

/// Helper to apply autosuggest options to query params.
fn apply_autosuggest_options(opts: &HereAutosuggestOptions, params: &mut Vec<(&str, String)>) {
    if let Some(at) = &opts.at {
        params.push(("at", format!("{},{}", at.lat, at.lng)));
    }
    if let Some(in_filter) = &opts.in_filter {
        params.push(("in", in_filter.clone()));
    }
    if let Some(lang) = &opts.lang {
        params.push(("lang", lang.clone()));
    }
    if let Some(limit) = opts.limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(pv) = &opts.political_view {
        params.push(("politicalView", pv.clone()));
    }
    if let Some(types) = &opts.types {
        let val = types.iter()
            .map(|t| serde_json::to_value(t).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("types", val));
    }
    if let Some(with) = &opts.with {
        let val = with.iter()
            .map(|w| serde_json::to_value(w).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("with", val));
    }
    if let Some(show) = &opts.show {
        let val = show.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("show", val));
    }
    if let Some(smr) = &opts.show_map_references {
        let val = smr.iter()
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.push(("showMapReferences", val));
    }
    if let Some(mobility) = &opts.mobility_mode {
        let val = serde_json::to_value(mobility).unwrap().as_str().unwrap().to_string();
        params.push(("mobilityMode", val));
    }
    if let Some(ranking) = &opts.ranking {
        let val = serde_json::to_value(ranking).unwrap().as_str().unwrap().to_string();
        params.push(("ranking", val));
    }
    if let Some(terms_limit) = opts.terms_limit {
        params.push(("termsLimit", terms_limit.to_string()));
    }
    if let Some(offset) = opts.offset {
        params.push(("offset", offset.to_string()));
    }
}

fn convert_geocode_item(item: HereGeocodeItem) -> SearchResult {
    let coordinate = item.position
        .map(Coordinate::from)
        .unwrap_or_else(|| Coordinate::new(0.0, 0.0).unwrap());
    let address = item.address
        .map(|a| Address {
            label: a.label,
            street: a.street,
            house_number: a.house_number,
            city: a.city,
            district: a.district,
            sub_district: a.sub_district,
            state: a.state,
            state_code: a.state_code,
            postal_code: a.postal_code,
            country: a.country_name,
            country_code: a.country_code,
            county: a.county,
            building: a.building,
            block: a.block,
            unit: a.unit,
        })
        .unwrap_or_default();
    let result_type = match item.result_type.as_deref() {
        Some("place") | Some("exactMatch") => SearchResultType::ExactMatch,
        Some("approximate") => SearchResultType::Approximate,
        Some("interpolated") => SearchResultType::Interpolated,
        _ => SearchResultType::Unknown,
    };
    SearchResult {
        id: item.id,
        coordinate,
        address,
        title: item.title,
        result_type,
        distance: item.distance,
        confidence: None,
        categories: vec![],
        bounding_box: None,
        raw: None,
    }
}

#[async_trait]
impl Geocoder for HereGeocoder {
    type Options = HereGeocodeOptions;
    type Response = SearchResponse;

    async fn geocode(&self, req: GeocodeRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if !req.query.is_empty() {
            params.push(("q", req.query));
        }

        apply_geocode_options(&req.options, &mut params);

        let url = format!("{}/geocode", self.geocode_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereGeocodeApiResponse = response.json().await?;

        let items = here_res.items.into_iter().map(convert_geocode_item).collect();

        Ok(SearchResponse { items })
    }

    async fn reverse_geocode(&self, req: ReverseGeocodeRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let mut params: Vec<(&str, String)> = vec![
            ("at", format!("{},{}", req.coordinate.lat, req.coordinate.lng)),
        ];

        apply_geocode_options(&req.options, &mut params);

        let url = format!("{}/reverseGeocode", self.geocode_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereGeocodeApiResponse = response.json().await?;

        let items = here_res.items.into_iter().map(convert_geocode_item).collect();

        Ok(SearchResponse { items })
    }
}