pub mod types;

use crate::client::RadarClient;
use crate::endpoints::{
    endpoint, ADDRESS_VALIDATION_PATH, AUTOCOMPLETE_PATH, FORWARD_GEOCODE_PATH, IP_GEOCODE_PATH,
    PLACE_SEARCH_PATH, RADAR_API_ROOT, REVERSE_GEOCODE_PATH,
};
use crate::error_mapping::{ensure_success, invalid_response};
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchQuality, SearchResponse, SearchResult,
    SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Address, Coordinate};
pub use types::*;

/// Implementation of `Geocoder` for Radar Geocoding API.
pub struct RadarGeocoder {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) service_root: String,
}

impl RadarGeocoder {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            service_root: RADAR_API_ROOT.to_string(),
        }
    }

    /// Creates a geocoder against a Radar service root.
    ///
    /// # Parameters
    /// - `client` — the shared Radar client.
    /// - `service_root` — the scheme and host serving the API, for example
    ///   `https://api.radar.io` or a test server's origin. Every Radar
    ///   endpoint is derived from this one root.
    pub fn with_base_url(client: std::sync::Arc<RadarClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The forward geocoding endpoint.
    pub(crate) fn forward_endpoint(&self) -> String {
        endpoint(&self.service_root, FORWARD_GEOCODE_PATH)
    }

    /// The reverse geocoding endpoint.
    pub(crate) fn reverse_endpoint(&self) -> String {
        endpoint(&self.service_root, REVERSE_GEOCODE_PATH)
    }

    /// The IP geocoding endpoint.
    pub(crate) fn ip_geocode_endpoint(&self) -> String {
        endpoint(&self.service_root, IP_GEOCODE_PATH)
    }

    /// The autocomplete endpoint.
    pub(crate) fn autocomplete_endpoint(&self) -> String {
        endpoint(&self.service_root, AUTOCOMPLETE_PATH)
    }

    /// The address validation endpoint.
    pub(crate) fn address_validation_endpoint(&self) -> String {
        endpoint(&self.service_root, ADDRESS_VALIDATION_PATH)
    }

    /// The place search endpoint.
    pub(crate) fn place_search_endpoint(&self) -> String {
        endpoint(&self.service_root, PLACE_SEARCH_PATH)
    }
}

impl TryFrom<RadarAddress> for SearchResult {
    type Error = EveryMapError;

    fn try_from(addr: RadarAddress) -> Result<Self, Self::Error> {
        let latitude = addr
            .latitude
            .ok_or_else(|| invalid_response("geocode result lacks a latitude"))?;
        let longitude = addr
            .longitude
            .ok_or_else(|| invalid_response("geocode result lacks a longitude"))?;
        let coordinate = Coordinate::new(latitude, longitude)
            .map_err(|_| invalid_response("geocode result carries an invalid location"))?;

        let confidence = addr.confidence.as_ref().map(|c| match c.as_str() {
            "exact" => 1.0,
            "interpolated" => 0.7,
            "fallback" => 0.3,
            _ => 0.5,
        });

        let result_type = match addr.confidence.as_deref() {
            Some("exact") => SearchResultType::ExactMatch,
            Some("interpolated") => SearchResultType::Interpolated,
            Some("fallback") => SearchResultType::Approximate,
            _ => SearchResultType::Unknown,
        };

        let address = Address {
            label: if addr.formatted_address.is_empty() {
                None
            } else {
                Some(addr.formatted_address.clone())
            },
            street: if addr.street.is_empty() {
                None
            } else {
                Some(addr.street.clone())
            },
            house_number: if addr.number.is_empty() {
                None
            } else {
                Some(addr.number.clone())
            },
            city: if addr.city.is_empty() {
                None
            } else {
                Some(addr.city.clone())
            },
            district: addr.neighborhood.clone(),
            state: if addr.state.is_empty() {
                None
            } else {
                Some(addr.state.clone())
            },
            state_code: if addr.state_code.is_empty() {
                None
            } else {
                Some(addr.state_code.clone())
            },
            postal_code: if addr.postal_code.is_empty() {
                None
            } else {
                Some(addr.postal_code.clone())
            },
            country: if addr.country.is_empty() {
                None
            } else {
                Some(addr.country.clone())
            },
            country_code: if addr.country_code.is_empty() {
                None
            } else {
                Some(addr.country_code.clone())
            },
            county: if addr.county.is_empty() {
                None
            } else {
                Some(addr.county.clone())
            },
            ..Address::default()
        };

        let title = if addr.formatted_address.is_empty() {
            None
        } else {
            Some(addr.formatted_address.clone())
        };
        let distance = addr.distance;
        let quality = addr.layer.as_deref().and_then(quality_from_layer);

        Ok(Self {
            id: None,
            title,
            coordinate,
            address,
            result_type,
            quality,
            distance,
            confidence,
            categories: vec![],
            bounding_box: None,
            raw: serde_json::to_value(&addr).ok(),
        })
    }
}

/// Maps a Radar address layer onto the contract's match quality.
fn quality_from_layer(layer: &str) -> Option<SearchQuality> {
    match layer.to_ascii_lowercase().as_str() {
        "venue" | "poi" => Some(SearchQuality::Point),
        "address" => Some(SearchQuality::StreetAddress),
        "street" => Some(SearchQuality::Street),
        "locality" | "city" => Some(SearchQuality::City),
        "county" => Some(SearchQuality::County),
        "state" => Some(SearchQuality::State),
        "country" => Some(SearchQuality::Country),
        "postalcode" | "postal_code" => Some(SearchQuality::PostalCode),
        _ => None,
    }
}

#[async_trait]
impl Geocoder for RadarGeocoder {
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("lang", lang.clone()));
        }
        if !options.country_codes.is_empty() {
            params.push(("country", options.country_codes.join(",")));
        }
        if options.bounding_box.is_some() {
            log::warn!(
                "Radar Geocoding API does not support bounding_box; \
                 bounding_box will be ignored"
            );
        }
        // Extract Radar-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("layers").and_then(|v| v.as_str()) {
                    params.push(("layers", v.to_string()));
                }
            }
        }

        let url = self.forward_endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let radar_res: RadarGeocodeResponse = self.client.request_parsed_json(builder).await?;
        ensure_success(radar_res.meta.code)?;

        let items: Vec<SearchResult> = radar_res
            .addresses
            .into_iter()
            .map(SearchResult::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;

        Ok(SearchResponse { items })
    }

    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse> {
        let mut params: Vec<(&str, String)> = vec![(
            "coordinates",
            format!("{},{}", coordinate.lat, coordinate.lng),
        )];

        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("lang", lang.clone()));
        }
        if let Some(radius) = options.radius {
            params.push(("radius", radius.to_string()));
        }
        // Extract Radar-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("layers").and_then(|v| v.as_str()) {
                    params.push(("layers", v.to_string()));
                }
            }
        }

        let url = self.reverse_endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let radar_res: RadarGeocodeResponse = self.client.request_parsed_json(builder).await?;
        ensure_success(radar_res.meta.code)?;

        let items: Vec<SearchResult> = radar_res
            .addresses
            .into_iter()
            .map(SearchResult::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;

        Ok(SearchResponse { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use everymap_core::domains::search::SearchResultType;

    fn sample_address() -> RadarAddress {
        RadarAddress {
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
            geometry: None,
            country: "US".to_string(),
            country_code: "US".to_string(),
            county: "New York".to_string(),
            confidence: Some("exact".to_string()),
            borough: Some("Manhattan".to_string()),
            city: "New York".to_string(),
            number: "350".to_string(),
            neighborhood: Some("Midtown".to_string()),
            postal_code: "10018".to_string(),
            state_code: "NY".to_string(),
            state: "New York".to_string(),
            street: "5th Avenue".to_string(),
            layer: None,
            formatted_address: "350 5th Avenue, New York, NY 10018".to_string(),
            address_label: None,
            time_zone: None,
            distance: Some(0.5),
            place_label: None,
        }
    }

    #[test]
    fn test_radar_address_to_search_result_exact_confidence() {
        let addr = sample_address();
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!((result.confidence.unwrap() - 1.0).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::ExactMatch));
        assert_eq!(
            result.title.as_deref(),
            Some("350 5th Avenue, New York, NY 10018")
        );
        assert!((result.coordinate.lat - 40.7128).abs() < f64::EPSILON);
        assert!((result.coordinate.lng - (-74.006)).abs() < f64::EPSILON);
        assert_eq!(result.address.street.as_deref(), Some("5th Avenue"));
        assert_eq!(result.address.house_number.as_deref(), Some("350"));
        assert_eq!(result.address.city.as_deref(), Some("New York"));
        assert_eq!(result.address.district.as_deref(), Some("Midtown"));
        assert_eq!(result.address.state.as_deref(), Some("New York"));
        assert_eq!(result.address.state_code.as_deref(), Some("NY"));
        assert_eq!(result.address.postal_code.as_deref(), Some("10018"));
        assert_eq!(result.address.country.as_deref(), Some("US"));
        assert_eq!(result.address.country_code.as_deref(), Some("US"));
        assert_eq!(result.address.county.as_deref(), Some("New York"));
        assert!((result.distance.unwrap() - 0.5).abs() < f64::EPSILON);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_radar_address_to_search_result_interpolated() {
        let mut addr = sample_address();
        addr.confidence = Some("interpolated".to_string());
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!((result.confidence.unwrap() - 0.7).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Interpolated));
    }

    #[test]
    fn test_radar_address_to_search_result_fallback() {
        let mut addr = sample_address();
        addr.confidence = Some("fallback".to_string());
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!((result.confidence.unwrap() - 0.3).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Approximate));
    }

    #[test]
    fn test_radar_address_to_search_result_unknown_confidence() {
        let mut addr = sample_address();
        addr.confidence = Some("unknown".to_string());
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!((result.confidence.unwrap() - 0.5).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Unknown));
    }

    #[test]
    fn test_radar_address_to_search_result_no_confidence() {
        let mut addr = sample_address();
        addr.confidence = None;
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!(result.confidence.is_none());
        assert!(matches!(result.result_type, SearchResultType::Unknown));
    }

    #[test]
    fn test_radar_address_empty_fields() {
        let addr = RadarAddress {
            latitude: Some(0.0),
            longitude: Some(0.0),
            geometry: None,
            country: String::new(),
            country_code: String::new(),
            county: String::new(),
            confidence: None,
            borough: None,
            city: String::new(),
            number: String::new(),
            neighborhood: None,
            postal_code: String::new(),
            state_code: String::new(),
            state: String::new(),
            street: String::new(),
            layer: None,
            formatted_address: String::new(),
            address_label: None,
            time_zone: None,
            distance: None,
            place_label: None,
        };
        let result = SearchResult::try_from(addr).expect("the address converts");

        assert!(result.title.is_none());
        assert!(result.address.label.is_none());
        assert!(result.address.street.is_none());
        assert!(result.address.house_number.is_none());
        assert!(result.address.city.is_none());
        assert!(result.address.state.is_none());
        assert!(result.address.state_code.is_none());
        assert!(result.address.postal_code.is_none());
        assert!(result.address.country.is_none());
        assert!(result.address.country_code.is_none());
        assert!(result.address.county.is_none());
        assert!(result.distance.is_none());
    }

    #[test]
    fn a_result_without_a_coordinate_is_invalid() {
        let mut addr = sample_address();
        addr.latitude = None;
        match SearchResult::try_from(addr) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn quality_comes_from_the_address_layer() {
        let mut addr = sample_address();
        addr.layer = Some("address".to_string());
        let result = SearchResult::try_from(addr).expect("the address converts");
        assert_eq!(result.quality, Some(SearchQuality::StreetAddress));
    }
}
