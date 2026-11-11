pub mod types;

use crate::client::{endpoint_url, GoogleClient};
use crate::error_mapping::{invalid_response, status_error};
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchQuality, SearchResponse, SearchResult,
    SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Address, BoundingBox, Coordinate};
pub use types::*;

/// Service root of the Google Maps Geocoding API.
const GEOCODING_SERVICE_ROOT: &str = "https://maps.googleapis.com";

/// Documented path of the Google Maps Geocoding API.
const GEOCODING_PATH: &str = "/maps/api/geocode/json";

/// Implementation of `Geocoder` for Google Maps Geocoding API.
pub struct GoogleGeocoder {
    pub(crate) client: std::sync::Arc<GoogleClient>,
    pub(crate) service_root: String,
}

impl GoogleGeocoder {
    pub fn new(client: std::sync::Arc<GoogleClient>) -> Self {
        Self {
            client,
            service_root: GEOCODING_SERVICE_ROOT.to_string(),
        }
    }

    /// Creates a geocoder against a Google service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    /// - `service_root` — the scheme and host that serve the API, for example
    ///   `https://maps.googleapis.com` or a test server's origin. The
    ///   documented path is appended.
    pub fn with_base_url(client: std::sync::Arc<GoogleClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The endpoint URL this geocoder sends requests to.
    fn endpoint(&self) -> String {
        endpoint_url(&self.service_root, GEOCODING_PATH)
    }
}

impl TryFrom<GoogleGeocodeResult> for SearchResult {
    type Error = EveryMapError;

    fn try_from(result: GoogleGeocodeResult) -> Result<Self, Self::Error> {
        let geometry = result
            .geometry
            .as_ref()
            .ok_or_else(|| invalid_response("geocode result lacks geometry"))?;
        let location = geometry
            .location
            .as_ref()
            .ok_or_else(|| invalid_response("geocode result lacks a location"))?;
        let coordinate = Coordinate::new(location.lat, location.lng)
            .map_err(|_| invalid_response("geocode result carries an invalid location"))?;

        let bounding_box = match geometry.viewport.as_ref() {
            Some(viewport) => {
                let north_east = Coordinate::new(viewport.northeast.lat, viewport.northeast.lng)
                    .map_err(|_| invalid_response("geocode viewport carries an invalid corner"))?;
                let south_west = Coordinate::new(viewport.southwest.lat, viewport.southwest.lng)
                    .map_err(|_| invalid_response("geocode viewport carries an invalid corner"))?;
                Some(BoundingBox::new(north_east, south_west))
            }
            None => None,
        };

        let address = Address {
            label: result.formatted_address.clone(),
            street: extract_component(&result.address_components, "street_number", "route"),
            city: extract_component_long(&result.address_components, "locality").or_else(|| {
                extract_component_long(&result.address_components, "administrative_area_level_2")
            }),
            state: extract_component_long(
                &result.address_components,
                "administrative_area_level_1",
            ),
            country_code: extract_component_short(&result.address_components, "country"),
            postal_code: extract_component_long(&result.address_components, "postal_code"),
            district: extract_component_long(&result.address_components, "sublocality"),
            house_number: extract_component_long(&result.address_components, "street_number"),
            ..Address::default()
        };

        let result_type = classify_result_type(&result.types);
        let quality = classify_quality(&result.types, geometry.location_type.as_deref());

        Ok(Self {
            id: result.place_id.clone(),
            title: result.formatted_address.clone(),
            coordinate,
            address,
            result_type,
            quality,
            distance: None,
            confidence: None,
            categories: vec![],
            bounding_box,
            raw: serde_json::to_value(&result).ok(),
        })
    }
}

/// Maps a Google address-component type onto the contract's match quality.
///
/// Google orders its `types` array from the most specific classification to
/// the most general, so the first recognised type wins.
fn classify_quality(types: &[String], location_type: Option<&str>) -> Option<SearchQuality> {
    types
        .iter()
        .find_map(|component_type| quality_from_component_type(component_type))
        .or_else(|| location_type.and_then(quality_from_location_type))
}

/// Maps one Google address-component type onto a match quality.
fn quality_from_component_type(component_type: &str) -> Option<SearchQuality> {
    match component_type {
        "street_address" | "premise" | "subpremise" | "postal_code_suffix" => {
            Some(SearchQuality::StreetAddress)
        }
        "route" => Some(SearchQuality::Street),
        "intersection" => Some(SearchQuality::Intersection),
        "postal_code" => Some(SearchQuality::PostalCode),
        "locality" | "postal_town" | "sublocality" | "sublocality_level_1" | "neighborhood" => {
            Some(SearchQuality::City)
        }
        "administrative_area_level_2" => Some(SearchQuality::County),
        "administrative_area_level_1" => Some(SearchQuality::State),
        "country" => Some(SearchQuality::Country),
        "point_of_interest" | "establishment" => Some(SearchQuality::Point),
        _ => None,
    }
}

/// Maps a Google `location_type` onto a match quality.
fn quality_from_location_type(location_type: &str) -> Option<SearchQuality> {
    match location_type {
        "ROOFTOP" => Some(SearchQuality::Point),
        "RANGE_INTERPOLATED" => Some(SearchQuality::StreetAddress),
        "GEOMETRIC_CENTER" => Some(SearchQuality::Street),
        "APPROXIMATE" => Some(SearchQuality::City),
        _ => None,
    }
}

fn classify_result_type(types: &[String]) -> SearchResultType {
    if types
        .iter()
        .any(|t| t == "street_address" || t == "premise")
    {
        SearchResultType::ExactMatch
    } else if types.iter().any(|t| t == "route" || t == "intersection") {
        SearchResultType::Approximate
    } else if types.iter().any(|t| t == "political" || t == "locality") {
        SearchResultType::Interpolated
    } else {
        SearchResultType::Unknown
    }
}

/// Extract a formatted street address from components.
fn extract_component(
    components: &[GoogleAddressComponent],
    number_type: &str,
    street_type: &str,
) -> Option<String> {
    let number = components
        .iter()
        .find(|c| c.types.iter().any(|t| t == number_type))
        .and_then(|c| c.long_name.clone());
    let street = components
        .iter()
        .find(|c| c.types.iter().any(|t| t == street_type))
        .and_then(|c| c.long_name.clone());
    match (number, street) {
        (Some(n), Some(s)) => Some(format!("{} {}", n, s)),
        (None, Some(s)) => Some(s),
        (Some(n), None) => Some(n),
        (None, None) => None,
    }
}

fn extract_component_long(
    components: &[GoogleAddressComponent],
    component_type: &str,
) -> Option<String> {
    components
        .iter()
        .find(|c| c.types.iter().any(|t| t == component_type))
        .and_then(|c| c.long_name.clone())
}

fn extract_component_short(
    components: &[GoogleAddressComponent],
    component_type: &str,
) -> Option<String> {
    components
        .iter()
        .find(|c| c.types.iter().any(|t| t == component_type))
        .and_then(|c| c.short_name.clone())
}

#[async_trait]
impl Geocoder for GoogleGeocoder {
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> = vec![("address", query.to_string())];

        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if let Some(bbox) = &options.bounding_box {
            params.push((
                "bounds",
                format!(
                    "{},{}|{},{}",
                    bbox.south_west.lat,
                    bbox.south_west.lng,
                    bbox.north_east.lat,
                    bbox.north_east.lng
                ),
            ));
        }
        if !options.country_codes.is_empty() {
            let components = options
                .country_codes
                .iter()
                .map(|code| format!("country:{}", code.to_lowercase()))
                .collect::<Vec<_>>()
                .join("|");
            params.push(("components", components));
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("region").and_then(|v| v.as_str()) {
                    params.push(("region", v.to_string()));
                }
                if let Some(v) = obj.get("components").and_then(|v| v.as_str()) {
                    params.push(("components", v.to_string()));
                }
            }
        }

        let url = self.endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let google_res: GoogleGeocodeResponse = self.client.request_parsed_json(builder).await?;

        if google_res.status != "OK" && google_res.status != "ZERO_RESULTS" {
            return Err(status_error(
                &google_res.status,
                google_res.error_message.as_deref(),
            ));
        }

        let mut items: Vec<SearchResult> = google_res
            .results
            .into_iter()
            .map(SearchResult::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;

        // Google API lacks a limit parameter; truncate client-side
        if let Some(limit) = options.limit {
            items.truncate(limit as usize);
        }

        Ok(SearchResponse { items })
    }

    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> =
            vec![("latlng", format!("{},{}", coordinate.lat, coordinate.lng))];

        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if options.limit.is_some() {
            log::warn!(
                "Google Reverse Geocoding API does not support a limit parameter; \
                 limit will be ignored"
            );
        }
        if options.radius.is_some() {
            log::warn!(
                "Google Reverse Geocoding API does not support a radius parameter; \
                 radius will be ignored"
            );
        }
        // Extract Google-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("result_type").and_then(|v| v.as_str()) {
                    params.push(("result_type", v.to_string()));
                }
                if let Some(v) = obj.get("location_type").and_then(|v| v.as_str()) {
                    params.push(("location_type", v.to_string()));
                }
            }
        }

        let url = self.endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let google_res: GoogleGeocodeResponse = self.client.request_parsed_json(builder).await?;

        if google_res.status != "OK" && google_res.status != "ZERO_RESULTS" {
            return Err(status_error(
                &google_res.status,
                google_res.error_message.as_deref(),
            ));
        }

        let items: Vec<SearchResult> = google_res
            .results
            .into_iter()
            .map(SearchResult::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;

        Ok(SearchResponse { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn deserialize_result(value: serde_json::Value) -> GoogleGeocodeResult {
        serde_json::from_value(value).expect("the fixture deserializes")
    }

    #[test]
    fn a_result_without_a_location_is_invalid() {
        let result = deserialize_result(json!({
            "formatted_address": "Berlin",
            "types": ["locality"]
        }));
        match SearchResult::try_from(result) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn quality_comes_from_the_most_specific_component_type() {
        let result = deserialize_result(json!({
            "formatted_address": "Brandenburger Tor, Berlin",
            "geometry": {
                "location": { "lat": 52.5163, "lng": 13.3777 },
                "location_type": "APPROXIMATE"
            },
            "types": ["street_address", "political"],
            "address_components": []
        }));
        let mapped = SearchResult::try_from(result).expect("the fixture maps");
        assert_eq!(mapped.quality, Some(SearchQuality::StreetAddress));
    }

    #[test]
    fn quality_falls_back_to_the_location_type() {
        let result = deserialize_result(json!({
            "formatted_address": "Some place",
            "geometry": {
                "location": { "lat": 52.5163, "lng": 13.3777 },
                "location_type": "ROOFTOP"
            },
            "types": ["political"],
            "address_components": []
        }));
        let mapped = SearchResult::try_from(result).expect("the fixture maps");
        assert_eq!(mapped.quality, Some(SearchQuality::Point));
    }
}
