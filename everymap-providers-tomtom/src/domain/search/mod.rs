pub mod types;

use crate::client::TomTomClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchQuality, SearchResponse, SearchResult,
    SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
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

/// Maps a TomTom result type onto the contract's match quality.
fn quality_from_result_type(result_type: &str) -> Option<SearchQuality> {
    match result_type.to_lowercase().as_str() {
        "point address" | "address range" | "address" => Some(SearchQuality::StreetAddress),
        "street" => Some(SearchQuality::Street),
        "cross street" => Some(SearchQuality::Intersection),
        "poi" => Some(SearchQuality::Point),
        // TomTom reports a settlement or administrative area as one coarse
        // "Geography" type; the contract's settlement level is the closest
        // match.
        "geography" => Some(SearchQuality::City),
        _ => None,
    }
}

/// Parses a TomTom `"lat,lon"` string into a coordinate.
fn parse_lat_lon(value: &str) -> Option<Coordinate> {
    let (latitude, longitude) = value.split_once(',')?;
    let latitude = latitude.trim().parse::<f64>().ok()?;
    let longitude = longitude.trim().parse::<f64>().ok()?;
    Coordinate::new(latitude, longitude).ok()
}

/// Infers the granularity of a reverse geocode result from its address.
///
/// A reverse geocode response carries no result type, so the most specific
/// populated address component names the granularity.
fn quality_from_reverse_address(address: &TomTomReverseGeocodeAddress) -> Option<SearchQuality> {
    if address.street_name.is_some() || address.street.is_some() {
        return Some(SearchQuality::StreetAddress);
    }
    if address.postal_code.is_some() {
        return Some(SearchQuality::PostalCode);
    }
    if address.municipality.is_some() || address.local_name.is_some() {
        return Some(SearchQuality::City);
    }
    if address.country_subdivision.is_some() {
        return Some(SearchQuality::State);
    }
    if address.country.is_some() {
        return Some(SearchQuality::Country);
    }
    None
}

impl TryFrom<TomTomSearchResult> for SearchResult {
    type Error = EveryMapError;

    fn try_from(r: TomTomSearchResult) -> Result<Self, Self::Error> {
        let position = r
            .position
            .as_ref()
            .ok_or_else(|| invalid_response("search result lacks a position"))?;
        let latitude = position
            .lat
            .ok_or_else(|| invalid_response("search result position lacks a latitude"))?;
        let longitude = position
            .lon
            .ok_or_else(|| invalid_response("search result position lacks a longitude"))?;
        let coordinate = Coordinate::new(latitude, longitude)
            .map_err(|_| invalid_response("search result carries an invalid position"))?;
        let quality = r.result_type.as_deref().and_then(quality_from_result_type);

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
                Coordinate::new(bounding_box.top_left.lat?, bounding_box.top_left.lon?).ok()?,
                Coordinate::new(bounding_box.btm_right.lat?, bounding_box.btm_right.lon?).ok()?,
            ))
        });

        let result_type = r
            .result_type
            .as_deref()
            .map(classify_result_type)
            .unwrap_or(SearchResultType::Unknown);

        Ok(Self {
            id: r.id,
            title,
            coordinate,
            address,
            result_type,
            quality,
            distance: r.total_distance,
            confidence: r.score,
            categories: Vec::new(),
            bounding_box,
            raw: None,
        })
    }
}

impl TryFrom<TomTomReverseGeocodeResult> for SearchResult {
    type Error = EveryMapError;

    fn try_from(r: TomTomReverseGeocodeResult) -> Result<Self, Self::Error> {
        // Reverse geocode position is a "lat,lon" string.
        let position = r
            .position
            .as_deref()
            .ok_or_else(|| invalid_response("reverse geocode result lacks a position"))?;
        let coordinate = parse_lat_lon(position).ok_or_else(|| {
            invalid_response("reverse geocode result carries an invalid position")
        })?;

        // Extract bounding_box before moving address.
        let bounding_box = r.address.as_ref().and_then(|a| {
            a.bounding_box.as_ref().and_then(|bb| {
                let north_east = bb.north_east.as_deref().and_then(parse_lat_lon)?;
                let south_west = bb.south_west.as_deref().and_then(parse_lat_lon)?;
                Some(BoundingBox::new(north_east, south_west))
            })
        });

        let title = r.address.as_ref().and_then(|a| a.freeform_address.clone());
        let quality = r.address.as_ref().and_then(quality_from_reverse_address);

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

        Ok(Self {
            id: r.id,
            title,
            coordinate,
            address,
            result_type: SearchResultType::ExactMatch,
            quality,
            distance: r.total_distance,
            confidence: None,
            categories: Vec::new(),
            bounding_box,
            raw: None,
        })
    }
}

impl TryFrom<TomTomSearchResponse> for SearchResponse {
    type Error = EveryMapError;

    fn try_from(response: TomTomSearchResponse) -> Result<Self, Self::Error> {
        // Forward geocode uses "results", reverse geocode uses "addresses".
        let items = if !response.results.is_empty() {
            response
                .results
                .into_iter()
                .map(SearchResult::try_from)
                .collect::<EveryMapResult<Vec<_>>>()?
        } else {
            response
                .addresses
                .into_iter()
                .map(SearchResult::try_from)
                .collect::<EveryMapResult<Vec<_>>>()?
        };
        Ok(SearchResponse { items })
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: TomTomSearchResponse = self.client.request_parsed_json(builder).await?;
        SearchResponse::try_from(result)
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: TomTomSearchResponse = self.client.request_parsed_json(builder).await?;
        SearchResponse::try_from(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_result_without_a_position_is_invalid() {
        let result: TomTomSearchResult = serde_json::from_value(json!({
            "address": { "freeformAddress": "Berlin" },
            "type": "Geography"
        }))
        .expect("the fixture deserializes");
        match SearchResult::try_from(result) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn quality_comes_from_the_result_type() {
        let result: TomTomSearchResult = serde_json::from_value(json!({
            "position": { "lat": 52.52, "lon": 13.405 },
            "type": "Point Address"
        }))
        .expect("the fixture deserializes");
        let mapped = SearchResult::try_from(result).expect("the fixture maps");
        assert_eq!(mapped.quality, Some(SearchQuality::StreetAddress));
    }

    #[test]
    fn reverse_quality_comes_from_the_most_specific_address_component() {
        let result: TomTomReverseGeocodeResult = serde_json::from_value(json!({
            "position": "52.520264,13.399690",
            "address": {
                "municipality": "Berlin",
                "country": "Deutschland"
            }
        }))
        .expect("the fixture deserializes");
        let mapped = SearchResult::try_from(result).expect("the fixture maps");
        assert_eq!(mapped.quality, Some(SearchQuality::City));
    }
}
