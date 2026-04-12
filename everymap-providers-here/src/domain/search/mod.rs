pub mod types;

use async_trait::async_trait;
use everymap_core::domains::search::{
    Geocoder, GeocodeOptions, ReverseGeocodeOptions,
    SearchResponse, SearchResult,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::HereClient;
use std::sync::Arc;

// Re-export all public types from the types module
pub use types::*;

const GEOCODE_BASE_URL: &str = "https://geocode.search.hereapi.com/v1";
const DISCOVER_BASE_URL: &str = "https://discover.search.hereapi.com/v1";
const AUTOSUGGEST_BASE_URL: &str = "https://autosuggest.search.hereapi.com/v1";

/// Exhaustive options for HERE Geocoding & Search API v7 geocode/reverseGeocode.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

/// Request wrapper for the HERE-specific discover endpoint.
pub struct DiscoverRequest {
    pub query: String,
    pub options: HereDiscoverOptions,
}

/// Request wrapper for the HERE-specific autosuggest endpoint.
pub struct AutosuggestRequest {
    pub query: String,
    pub options: HereAutosuggestOptions,
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
        req: DiscoverRequest,
    ) -> EveryMapResult<HereDiscoverResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();
        params.push(("q", req.query));
        apply_discover_options(&req.options, &mut params);

        let url = format!("{}/discover", self.discover_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereDiscoverResponse = self.client.request_json(builder).await?;

        Ok(here_res)
    }

    /// Get autosuggest results for a partial query (type-ahead).
    pub async fn autosuggest(
        &self,
        req: AutosuggestRequest,
    ) -> EveryMapResult<HereAutosuggestResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();
        params.push(("q", req.query));
        apply_autosuggest_options(&req.options, &mut params);

        let url = format!("{}/autosuggest", self.autosuggest_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereAutosuggestResponse = self.client.request_json(builder).await?;

        Ok(here_res)
    }
}

/// Convert core `GeocodeOptions` to HERE-specific `HereGeocodeOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn geocode_options_from_core(opts: &GeocodeOptions) -> HereGeocodeOptions {
    let mut here_opts = HereGeocodeOptions {
        limit: opts.limit,
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Convert bounding_box to HERE's `at` + `in_filter` convention if present
    if let Some(bb) = &opts.bounding_box {
        // Use center of bounding box as `at`
        if let Ok(center) = Coordinate::new(
            (bb.north_east.lat + bb.south_west.lat) / 2.0,
            (bb.north_east.lng + bb.south_west.lng) / 2.0,
        ) {
            here_opts.at = Some(center);
        }
        // Use bounding box as `in` filter
        here_opts.in_filter = Some(format!(
            "bbox:{},{},{},{}",
            bb.south_west.lng, bb.south_west.lat,
            bb.north_east.lng, bb.north_east.lat
        ));
    }

    // Convert country_codes to HERE's `in_filter` if not already set
    if !opts.country_codes.is_empty() {
        let countries = opts.country_codes.join(",");
        if let Some(existing) = &mut here_opts.in_filter {
            // Append country filter to existing in_filter
            *existing = format!("{}+countryCode:{}", existing, countries);
        } else {
            here_opts.in_filter = Some(format!("countryCode:{}", countries));
        }
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            extract_here_geocode_extra(obj, &mut here_opts);
        }
    }

    here_opts
}

/// Convert core `ReverseGeocodeOptions` to HERE-specific `HereGeocodeOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn reverse_geocode_options_from_core(opts: &ReverseGeocodeOptions) -> HereGeocodeOptions {
    let mut here_opts = HereGeocodeOptions {
        limit: opts.limit,
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            extract_here_geocode_extra(obj, &mut here_opts);
        }
    }

    here_opts
}

/// Extract HERE-specific fields from a JSON object into `HereGeocodeOptions`.
fn extract_here_geocode_extra(
    obj: &serde_json::Map<String, serde_json::Value>,
    here_opts: &mut HereGeocodeOptions,
) {
    if let Some(v) = obj.get("at") {
        if let Ok(coord) = serde_json::from_value::<Coordinate>(v.clone()) {
            here_opts.at = Some(coord);
        }
    }
    if let Some(v) = obj.get("in_filter").and_then(|v| v.as_str()) {
        here_opts.in_filter = Some(v.to_string());
    }
    if let Some(v) = obj.get("qq").and_then(|v| v.as_str()) {
        here_opts.qq = Some(v.to_string());
    }
    if let Some(v) = obj.get("lang").and_then(|v| v.as_str()) {
        here_opts.lang = Some(v.to_string());
    }
    if let Some(v) = obj.get("limit").and_then(|v| v.as_u64()) {
        here_opts.limit = Some(v as u32);
    }
    if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
        here_opts.political_view = Some(v.to_string());
    }
    if let Some(v) = obj.get("address_names_variant").and_then(|v| v.as_str()) {
        here_opts.address_names_variant = Some(v.to_string());
    }
    if let Some(v) = obj.get("request_id").and_then(|v| v.as_str()) {
        here_opts.request_id = Some(v.to_string());
    }
    if let Some(v) = obj.get("address_names_mode") {
        if let Ok(parsed) = serde_json::from_value::<AddressNamesMode>(v.clone()) {
            here_opts.address_names_mode = Some(parsed);
        }
    }
    if let Some(v) = obj.get("postal_code_mode") {
        if let Ok(parsed) = serde_json::from_value::<PostalCodeMode>(v.clone()) {
            here_opts.postal_code_mode = Some(parsed);
        }
    }
    if let Some(v) = obj.get("types").and_then(|v| v.as_array()) {
        let parsed: Vec<SearchType> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.types = Some(parsed);
        }
    }
    if let Some(v) = obj.get("with").and_then(|v| v.as_array()) {
        let parsed: Vec<WithFeature> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.with = Some(parsed);
        }
    }
    if let Some(v) = obj.get("show").and_then(|v| v.as_array()) {
        let parsed: Vec<ShowFeature> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.show = Some(parsed);
        }
    }
    if let Some(v) = obj.get("show_map_references").and_then(|v| v.as_array()) {
        let parsed: Vec<ShowMapReference> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.show_map_references = Some(parsed);
        }
    }
    if let Some(v) = obj.get("show_nav_attributes").and_then(|v| v.as_array()) {
        let parsed: Vec<ShowNavAttribute> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.show_nav_attributes = Some(parsed);
        }
    }
    if let Some(v) = obj.get("show_related").and_then(|v| v.as_array()) {
        let parsed: Vec<ShowRelated> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.show_related = Some(parsed);
        }
    }
    if let Some(v) = obj.get("show_translations").and_then(|v| v.as_array()) {
        let parsed: Vec<ShowTranslation> = v.iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        if !parsed.is_empty() {
            here_opts.show_translations = Some(parsed);
        }
    }
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

#[async_trait]
impl Geocoder for HereGeocoder {
    async fn geocode(&self, query: &str, options: &GeocodeOptions) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if !query.is_empty() {
            params.push(("q", query.to_string()));
        }

        let here_opts = geocode_options_from_core(options);
        apply_geocode_options(&here_opts, &mut params);

        let url = format!("{}/geocode", self.geocode_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereSearchResponse = self.client.request_json(builder).await?;

        let items = here_res.items.into_iter().map(SearchResult::from).collect();

        Ok(SearchResponse { items })
    }

    async fn reverse_geocode(&self, coordinate: &Coordinate, options: &ReverseGeocodeOptions) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("at", format!("{},{}", coordinate.lat, coordinate.lng)),
        ];

        let here_opts = reverse_geocode_options_from_core(options);
        apply_geocode_options(&here_opts, &mut params);

        let url = format!("{}/reverseGeocode", self.geocode_base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereSearchResponse = self.client.request_json(builder).await?;

        let items = here_res.items.into_iter().map(SearchResult::from).collect();

        Ok(SearchResponse { items })
    }
}