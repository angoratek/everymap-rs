pub mod types;

use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{Address, BoundingBox, Coordinate};
use std::sync::Arc;

pub use types::*;

const SEARCH_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of Geocoder for TomTom Search API.
pub struct TomTomGeocoder {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomGeocoder {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: SEARCH_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert TomTom result type string to core SearchResultType.
fn classify_result_type(result_type: &str) -> SearchResultType {
    match result_type.to_lowercase().as_str() {
        "point address" => SearchResultType::ExactMatch,
        "address" => SearchResultType::ExactMatch,
        "street" => SearchResultType::Approximate,
        "cross street" => SearchResultType::Approximate,
        "geography" => SearchResultType::Approximate,
        "poi" => SearchResultType::Approximate,
        _ => SearchResultType::Unknown,
    }
}

impl From<TomTomSearchResult> for SearchResult {
    fn from(r: TomTomSearchResult) -> Self {
        let coordinate = r
            .position
            .map(|p| Coordinate::new(p.lat, p.lon).unwrap_or(Coordinate::ORIGIN))
            .unwrap_or(Coordinate::ORIGIN);

        let title = r.address.as_ref().and_then(|a| a.freeform_address.clone());

        let address = r
            .address
            .map(|a| {
                let mut addr = Address::empty();
                addr.label = a.freeform_address;
                addr.street = a.street_name;
                addr.house_number = a.street_number;
                addr.city = a.municipality;
                addr.state = a.country_subdivision;
                addr.country = a.country;
                addr.country_code = a.country_code;
                addr.postal_code = a.postal_code;
                addr.district = a.neighbourhood;
                addr
            })
            .unwrap_or_else(Address::empty);

        let bounding_box = r.bounding_box.and_then(|bounding_box| {
            Some(BoundingBox::new(
                Coordinate::new(bounding_box.top_left.lat, bounding_box.top_left.lon).ok()?,
                Coordinate::new(bounding_box.btm_right.lat, bounding_box.btm_right.lon).ok()?,
            ))
        });

        let result_type = r
            .result_type
            .as_deref()
            .map(classify_result_type)
            .unwrap_or(SearchResultType::Unknown);

        SearchResult {
            id: r.id,
            title,
            coordinate,
            address,
            result_type,
            distance: r.total_distance,
            confidence: r.score,
            categories: Vec::new(),
            bounding_box,
            raw: None,
        }
    }
}

impl From<TomTomReverseGeocodeResult> for SearchResult {
    fn from(r: TomTomReverseGeocodeResult) -> Self {
        // Reverse geocode position is a "lat,lon" string
        let coordinate = r
            .position
            .as_deref()
            .and_then(|s| {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 2 {
                    let lat = parts[0].parse::<f64>().ok()?;
                    let lng = parts[1].parse::<f64>().ok()?;
                    Coordinate::new(lat, lng).ok()
                } else {
                    None
                }
            })
            .unwrap_or(Coordinate::ORIGIN);

        // Extract bounding_box before moving address
        let bounding_box = r.address.as_ref().and_then(|a| {
            a.bounding_box.as_ref().and_then(|bb| {
                fn parse_lat_lon(s: &str) -> Option<Coordinate> {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() == 2 {
                        let lat = parts[0].parse::<f64>().ok()?;
                        let lng = parts[1].parse::<f64>().ok()?;
                        Coordinate::new(lat, lng).ok()
                    } else {
                        None
                    }
                }
                let ne = bb.north_east.as_deref().and_then(parse_lat_lon)?;
                let sw = bb.south_west.as_deref().and_then(parse_lat_lon)?;
                Some(BoundingBox::new(ne, sw))
            })
        });

        let title = r.address.as_ref().and_then(|a| a.freeform_address.clone());

        let address = r
            .address
            .map(|a| {
                let mut addr = Address::empty();
                addr.label = a.freeform_address;
                addr.street = a.street_name.or(a.street);
                addr.city = a.municipality.or(a.local_name);
                addr.state = a.country_subdivision;
                addr.country = a.country;
                addr.country_code = a.country_code;
                addr.postal_code = a.postal_code;
                addr.district = a.neighbourhood.or(a.municipality_subdivision);
                addr
            })
            .unwrap_or_else(Address::empty);

        SearchResult {
            id: r.id,
            title,
            coordinate,
            address,
            result_type: SearchResultType::ExactMatch,
            distance: r.total_distance,
            confidence: None,
            categories: Vec::new(),
            bounding_box,
            raw: None,
        }
    }
}

impl From<TomTomSearchResponse> for SearchResponse {
    fn from(response: TomTomSearchResponse) -> Self {
        // Forward geocode uses "results", reverse geocode uses "addresses"
        let items = if !response.results.is_empty() {
            response.results.into_iter().map(|r| r.into()).collect()
        } else {
            response.addresses.into_iter().map(|r| r.into()).collect()
        };
        SearchResponse { items }
    }
}

#[async_trait]
impl Geocoder for TomTomGeocoder {
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let encoded_query = query.replace(' ', "+");
        let url = format!("{}/search/2/geocode/{}.json", self.base_url, encoded_query);
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if let Some(bbox) = &options.bounding_box {
            params.push((
                "bbox",
                format!(
                    "{},{},{},{}",
                    bbox.south_west.lng,
                    bbox.south_west.lat,
                    bbox.north_east.lng,
                    bbox.north_east.lat
                ),
            ));
        }
        if !options.country_codes.is_empty() {
            let codes: String = options.country_codes.join(",");
            params.push(("countrySet", codes));
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("typeahead").and_then(|v| v.as_bool()) {
                    params.push(("typeahead", v.to_string()));
                }
                if let Some(v) = obj.get("index").and_then(|v| v.as_str()) {
                    params.push(("index", v.to_string()));
                }
                if let Some(v) = obj.get("view").and_then(|v| v.as_str()) {
                    params.push(("view", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: TomTomSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }

    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let url = format!(
            "{}/search/2/reverseGeocode/{},{}.json",
            self.base_url, coordinate.lat, coordinate.lng
        );
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if let Some(radius) = options.radius {
            params.push(("radius", radius.to_string()));
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("view").and_then(|v| v.as_str()) {
                    params.push(("view", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: TomTomSearchResponse = self.client.request_json(builder).await?;
        Ok(result.into())
    }
}
