//! Geocoding against the MapQuest Geocoding API v1.
//!
//! The provider implements
//! [`Geocoder`] for the documented
//! address (`geocoding/v1/address`) and reverse (`geocoding/v1/reverse`)
//! endpoints. Both endpoints accept the location as a single `location` query
//! value and return `results[].locations[]`.
//!
//! The service's `geocodeQuality` vocabulary (and, as a fallback, the precision
//! letter of its five-character `geocodeQualityCode`) is projected onto the
//! core's three-way
//! [`SearchResultType`]:
//! precise point-like matches are [`ExactMatch`], street-level matches are
//! [`Approximate`], and area-level matches (postal code, settlement, county,
//! state, country) are [`Interpolated`].
//!
//! [`ExactMatch`]: everymap_core::domains::search::SearchResultType::ExactMatch
//! [`Approximate`]: everymap_core::domains::search::SearchResultType::Approximate
//! [`Interpolated`]: everymap_core::domains::search::SearchResultType::Interpolated

use crate::client::MapQuestClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Address, Coordinate};
use serde_json::Value;
use std::sync::Arc;

/// Service path of the forward (address) geocoding endpoint.
const ADDRESS_GEOCODING_PATH: &str = "geocoding/v1/address";

/// Service path of the reverse geocoding endpoint.
const REVERSE_GEOCODING_PATH: &str = "geocoding/v1/reverse";

/// Query parameter carrying the text or coordinate to geocode.
const LOCATION_PARAMETER: &str = "location";

/// Query parameter capping the number of returned locations.
const MAX_RESULTS_PARAMETER: &str = "maxResults";

/// Geocoding against the MapQuest Geocoding API v1.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`]. The
/// client supplies the base URL, credential and request pipeline; the provider
/// only builds the documented query and projects the answer.
pub struct MapQuestGeocoder {
    client: Arc<MapQuestClient>,
}

impl MapQuestGeocoder {
    /// Builds a geocoder around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A geocoder ready for forward and reverse requests.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Geocoder for MapQuestGeocoder {
    /// Resolves an address or place text to coordinates.
    ///
    /// # Parameters
    /// - `query` — the address or place text. A blank value is rejected
    ///   before any request is sent.
    /// - `options` — the result cap. The Geocoding API accepts
    ///   `maxResults`; language, country restriction, bounding box and
    ///   provider extras have no parameter on this endpoint and are reported
    ///   as unsupported through the log.
    ///
    /// # Returns
    /// One [`SearchResult`] per returned location, in service order.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when `query` is blank.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when the body is not JSON.
    /// - [`EveryMapError::ProviderError`] when the service reports a
    ///   non-zero `info.statuscode` or a payload lacks a required field.
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        if query.trim().is_empty() {
            return Err(EveryMapError::ValidationError(
                "address text must not be empty".to_string(),
            ));
        }
        let mut query_parameters = vec![(LOCATION_PARAMETER.to_string(), query.to_string())];
        if let Some(limit) = options.limit {
            query_parameters.push((MAX_RESULTS_PARAMETER.to_string(), limit.to_string()));
        }
        if options.language.is_some() {
            log::warn!(
                "MapQuest forward geocoding does not support a language parameter; ignoring"
            );
        }
        if !options.country_codes.is_empty() {
            log::warn!("MapQuest forward geocoding does not support country restriction; ignoring");
        }
        if options.bounding_box.is_some() {
            log::warn!("MapQuest forward geocoding does not support a bounding box; ignoring");
        }
        if options.provider_extra.is_some() {
            log::warn!("MapQuest forward geocoding does not consume provider_extra; ignoring");
        }
        let payload = self
            .client
            .get_json(ADDRESS_GEOCODING_PATH, &query_parameters)
            .await?;
        parse_geocoding_payload(&payload)
    }

    /// Resolves a coordinate to the addresses the service knows there.
    ///
    /// # Parameters
    /// - `coordinate` — the position to reverse geocode.
    /// - `options` — the result cap. The endpoint accepts `maxResults`; a
    ///   search radius, language and provider extras have no parameter here
    ///   and are reported as unsupported through the log.
    ///
    /// # Returns
    /// One [`SearchResult`] per returned location, in service order.
    ///
    /// # Errors
    /// Returns the same failures as [`Self::geocode`].
    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let location = format!("{},{}", coordinate.lat, coordinate.lng);
        let mut query_parameters = vec![(LOCATION_PARAMETER.to_string(), location)];
        if let Some(limit) = options.limit {
            query_parameters.push((MAX_RESULTS_PARAMETER.to_string(), limit.to_string()));
        }
        if options.radius.is_some() {
            log::warn!("MapQuest reverse geocoding does not support a radius; ignoring");
        }
        if options.language.is_some() {
            log::warn!(
                "MapQuest reverse geocoding does not support a language parameter; ignoring"
            );
        }
        if options.provider_extra.is_some() {
            log::warn!("MapQuest reverse geocoding does not consume provider_extra; ignoring");
        }
        let payload = self
            .client
            .get_json(REVERSE_GEOCODING_PATH, &query_parameters)
            .await?;
        parse_geocoding_payload(&payload)
    }
}

/// Projects a `results[].locations[]` geocoding payload into vendor-neutral
/// search results.
///
/// # Parameters
/// - `payload` — a decoded geocoding response body. It must carry a `results`
///   array; every result must carry a `locations` array.
///
/// # Returns
/// One [`SearchResult`] per location, in the order the service returned them.
///
/// # Errors
/// - [`EveryMapError::ProviderError`] when `results` or a `locations` array is
///   missing, or when a location carries no usable `latLng`.
pub fn parse_geocoding_payload(payload: &Value) -> EveryMapResult<SearchResponse> {
    let results = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("geocoding payload lacks results"))?;
    let mut items = Vec::new();
    for result in results {
        let locations = result
            .get("locations")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_response("geocoding result lacks locations"))?;
        for location in locations {
            items.push(project_location(location)?);
        }
    }
    Ok(SearchResponse { items })
}

/// Projects one `locations[]` entry into a [`SearchResult`].
fn project_location(location: &Value) -> EveryMapResult<SearchResult> {
    let latitude = location
        .pointer("/latLng/lat")
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_response("geocoding location lacks latLng"))?;
    let longitude = location
        .pointer("/latLng/lng")
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_response("geocoding location lacks latLng"))?;
    let coordinate = Coordinate::new(latitude, longitude)
        .map_err(|error| invalid_response(error.to_string()))?;

    let street = string_field(location, "street").unwrap_or_default();
    let city = string_field(location, "adminArea5").unwrap_or_default();
    let state_code = string_field(location, "adminArea3").unwrap_or_default();
    let county = string_field(location, "adminArea4");
    let district = string_field(location, "adminArea6");
    let country_code = string_field(location, "adminArea1");
    let postal_code = string_field(location, "postalCode");

    let label = [street.as_str(), city.as_str(), state_code.as_str()]
        .into_iter()
        .chain(postal_code.as_deref())
        .chain(country_code.as_deref())
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(", ");

    let address = Address {
        label: non_empty(label.clone()),
        street: non_empty(street),
        house_number: None,
        city: non_empty(city),
        district,
        sub_district: None,
        state: None,
        state_code: non_empty(state_code),
        postal_code,
        country: country_code.clone(),
        country_code,
        county,
        building: None,
        block: None,
        unit: None,
    };

    let quality_text = location
        .get("geocodeQuality")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let quality_code = location
        .get("geocodeQualityCode")
        .and_then(Value::as_str)
        .unwrap_or_default();

    Ok(SearchResult {
        id: None,
        coordinate,
        address,
        title: non_empty(label),
        result_type: map_geocode_quality(quality_text, quality_code),
        distance: None,
        confidence: None,
        categories: Vec::new(),
        bounding_box: None,
        raw: Some(location.clone()),
    })
}

/// Maps the documented `geocodeQuality` vocabulary to the core result type.
///
/// The fallback reads the precision letter of the five-character
/// `geocodeQualityCode` when `quality_text` is unknown.
///
/// # Parameters
/// - `quality_text` — the service's `geocodeQuality` value, such as `CITY`.
/// - `quality_code` — the service's `geocodeQualityCode` value, such as
///   `A5XAX`.
///
/// # Returns
/// - [`ExactMatch`](SearchResultType::ExactMatch) for `POINT`, `ADDRESS` and
///   `INTERSECTION`, and for precision letters `P`, `L`, `A` and `I`.
/// - [`Approximate`](SearchResultType::Approximate) for `STREET` and precision
///   letter `S`.
/// - [`Interpolated`](SearchResultType::Interpolated) for `ZIP`, `CITY`,
///   `COUNTY`, `STATE`, `COUNTRY`, `NEIGHBORHOOD` and precision letters `Z`,
///   `C`, `D`, `T` and `R`.
/// - [`Unknown`](SearchResultType::Unknown) for any other value.
pub fn map_geocode_quality(quality_text: &str, quality_code: &str) -> SearchResultType {
    match quality_text {
        "POINT" | "ADDRESS" | "INTERSECTION" => SearchResultType::ExactMatch,
        "STREET" => SearchResultType::Approximate,
        "ZIP" | "CITY" | "COUNTY" | "STATE" | "COUNTRY" | "NEIGHBORHOOD" => {
            SearchResultType::Interpolated
        }
        _ => quality_code
            .chars()
            .next()
            .map_or(SearchResultType::Unknown, map_quality_code_letter),
    }
}

/// Maps the precision letter of a `geocodeQualityCode` to the core result
/// type.
fn map_quality_code_letter(precision_letter: char) -> SearchResultType {
    match precision_letter {
        'P' | 'L' | 'A' | 'I' => SearchResultType::ExactMatch,
        'S' => SearchResultType::Approximate,
        'Z' | 'C' | 'D' | 'T' | 'R' => SearchResultType::Interpolated,
        _ => SearchResultType::Unknown,
    }
}

/// Reads a string field, returning `None` for an absent or empty value.
fn string_field(value: &Value, field_name: &str) -> Option<String> {
    value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .and_then(non_empty)
}

/// Converts an empty string to `None`.
fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{map_geocode_quality, parse_geocoding_payload};
    use everymap_core::domains::search::SearchResultType;

    fn fixture(name: &str) -> serde_json::Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    #[test]
    fn a_recorded_city_geocode_projects_its_coordinate_and_address() {
        let response = parse_geocoding_payload(&fixture("geocode_address_city.json"))
            .expect("the recorded payload must project");
        assert_eq!(response.items.len(), 1);
        let result = &response.items[0];
        assert_eq!(result.coordinate.lat, 38.57944);
        assert_eq!(result.coordinate.lng, -121.49085);
        assert_eq!(result.result_type, SearchResultType::Interpolated);
        assert_eq!(result.address.city.as_deref(), Some("Sacramento"));
        assert_eq!(result.address.state_code.as_deref(), Some("CA"));
        assert_eq!(result.address.country_code.as_deref(), Some("US"));
        assert!(result
            .address
            .label
            .as_deref()
            .is_some_and(|label| label.contains("Sacramento")));
        assert!(result.raw.is_some());
    }

    #[test]
    fn a_recorded_reverse_geocode_projects_the_point_quality() {
        let response = parse_geocoding_payload(&fixture("geocode_reverse_capitol.json"))
            .expect("the recorded payload must project");
        assert_eq!(response.items.len(), 1);
        let result = &response.items[0];
        assert_eq!(result.result_type, SearchResultType::ExactMatch);
        assert_eq!(result.address.street.as_deref(), Some("1315 10th St"));
        assert_eq!(result.address.city.as_deref(), Some("Sacramento"));
        assert_eq!(result.address.postal_code.as_deref(), Some("95814"));
        assert_eq!(result.address.district.as_deref(), Some("Capitol Mall"));
    }

    #[test]
    fn the_quality_vocabulary_maps_to_the_core_result_types() {
        assert_eq!(
            map_geocode_quality("POINT", ""),
            SearchResultType::ExactMatch
        );
        assert_eq!(
            map_geocode_quality("ADDRESS", ""),
            SearchResultType::ExactMatch
        );
        assert_eq!(
            map_geocode_quality("INTERSECTION", ""),
            SearchResultType::ExactMatch
        );
        assert_eq!(
            map_geocode_quality("STREET", ""),
            SearchResultType::Approximate
        );
        for area_quality in ["ZIP", "CITY", "COUNTY", "STATE", "COUNTRY", "NEIGHBORHOOD"] {
            assert_eq!(
                map_geocode_quality(area_quality, ""),
                SearchResultType::Interpolated,
                "{area_quality} is an area-level match"
            );
        }
        assert_eq!(map_geocode_quality("", "?9ZZZ"), SearchResultType::Unknown);
    }

    #[test]
    fn the_quality_code_letter_is_the_fallback() {
        assert_eq!(
            map_geocode_quality("", "P1CAA"),
            SearchResultType::ExactMatch
        );
        assert_eq!(
            map_geocode_quality("", "A5XAX"),
            SearchResultType::ExactMatch
        );
        assert_eq!(
            map_geocode_quality("", "S5XAX"),
            SearchResultType::Approximate
        );
        assert_eq!(
            map_geocode_quality("", "Z3AAA"),
            SearchResultType::Interpolated
        );
    }
}
