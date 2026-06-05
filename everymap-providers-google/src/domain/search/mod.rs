pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Address, BoundingBox, Coordinate};
pub use types::*;

const GEOCODING_BASE_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json";

/// Implementation of `Geocoder` for Google Maps Geocoding API.
pub struct GoogleGeocoder {
    pub(crate) client: std::sync::Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleGeocoder {
    pub fn new(client: std::sync::Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: GEOCODING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

impl From<GoogleGeocodeResult> for SearchResult {
    fn from(result: GoogleGeocodeResult) -> Self {
        let coordinate = result
            .geometry
            .as_ref()
            .and_then(|g| g.location.as_ref())
            .map(|loc| Coordinate::new(loc.lat, loc.lng).unwrap_or(Coordinate::ORIGIN))
            .unwrap_or(Coordinate::ORIGIN);

        let bounding_box = result
            .geometry
            .as_ref()
            .and_then(|g| g.viewport.as_ref())
            .map(|v| {
                BoundingBox::new(
                    Coordinate::new(v.northeast.lat, v.northeast.lng).unwrap_or(Coordinate::ORIGIN),
                    Coordinate::new(v.southwest.lat, v.southwest.lng).unwrap_or(Coordinate::ORIGIN),
                )
            });

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

        Self {
            id: result.place_id.clone(),
            title: result.formatted_address.clone(),
            coordinate,
            address,
            result_type,
            distance: None,
            confidence: None,
            categories: vec![],
            bounding_box,
            raw: Some(serde_json::to_value(result).unwrap_or_default()),
        }
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

        let url = self.base_url.clone();
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let google_res: GoogleGeocodeResponse = self.client.request_json(builder).await?;

        if google_res.status != "OK" && google_res.status != "ZERO_RESULTS" {
            return Err(EveryMapError::provider(
                "google",
                &google_res.status,
                google_res
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error"),
            ));
        }

        let mut items: Vec<SearchResult> = google_res
            .results
            .into_iter()
            .map(SearchResult::from)
            .collect();

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

        let url = self.base_url.clone();
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let google_res: GoogleGeocodeResponse = self.client.request_json(builder).await?;

        if google_res.status != "OK" && google_res.status != "ZERO_RESULTS" {
            return Err(EveryMapError::provider(
                "google",
                &google_res.status,
                google_res
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error"),
            ));
        }

        let items: Vec<SearchResult> = google_res
            .results
            .into_iter()
            .map(SearchResult::from)
            .collect();

        Ok(SearchResponse { items })
    }
}
