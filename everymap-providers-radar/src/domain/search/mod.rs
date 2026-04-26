pub mod types;

use crate::client::RadarClient;
use async_trait::async_trait;
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResponse, SearchResult, SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Address, Coordinate};
pub use types::*;

const GEOCODING_BASE_URL: &str = "https://api.radar.io/v1/geocode/forward";
const REVERSE_GEOCODING_BASE_URL: &str = "https://api.radar.io/v1/geocode/reverse";

/// Implementation of `Geocoder` for Radar Geocoding API.
pub struct RadarGeocoder {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
    pub(crate) reverse_base_url: String,
}

impl RadarGeocoder {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: GEOCODING_BASE_URL.to_string(),
            reverse_base_url: REVERSE_GEOCODING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(
        client: std::sync::Arc<RadarClient>,
        base_url: String,
        reverse_base_url: String,
    ) -> Self {
        Self {
            client,
            base_url,
            reverse_base_url,
        }
    }
}

impl From<RadarAddress> for SearchResult {
    fn from(addr: RadarAddress) -> Self {
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
        let coordinate =
            Coordinate::new(addr.latitude, addr.longitude).unwrap_or(Coordinate::ORIGIN);

        Self {
            id: None,
            title,
            coordinate,
            address,
            result_type,
            distance,
            confidence,
            categories: vec![],
            bounding_box: None,
            raw: Some(serde_json::to_value(addr).unwrap_or_default()),
        }
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
        // Extract Radar-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("layers").and_then(|v| v.as_str()) {
                    params.push(("layers", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.base_url)
            .query(&params);

        let radar_res: RadarGeocodeResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!(
                    "Geocoding request failed with status {}",
                    radar_res.meta.code
                ),
            ));
        }

        let items: Vec<SearchResult> = radar_res
            .addresses
            .into_iter()
            .map(SearchResult::from)
            .collect();

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

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.reverse_base_url)
            .query(&params);

        let radar_res: RadarGeocodeResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!(
                    "Reverse geocoding request failed with status {}",
                    radar_res.meta.code
                ),
            ));
        }

        let items: Vec<SearchResult> = radar_res
            .addresses
            .into_iter()
            .map(SearchResult::from)
            .collect();

        Ok(SearchResponse { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use everymap_core::domains::search::SearchResultType;

    fn sample_address() -> RadarAddress {
        RadarAddress {
            latitude: 40.7128,
            longitude: -74.0060,
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
        let result: SearchResult = addr.into();

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
        let result: SearchResult = addr.into();

        assert!((result.confidence.unwrap() - 0.7).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Interpolated));
    }

    #[test]
    fn test_radar_address_to_search_result_fallback() {
        let mut addr = sample_address();
        addr.confidence = Some("fallback".to_string());
        let result: SearchResult = addr.into();

        assert!((result.confidence.unwrap() - 0.3).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Approximate));
    }

    #[test]
    fn test_radar_address_to_search_result_unknown_confidence() {
        let mut addr = sample_address();
        addr.confidence = Some("unknown".to_string());
        let result: SearchResult = addr.into();

        assert!((result.confidence.unwrap() - 0.5).abs() < f64::EPSILON);
        assert!(matches!(result.result_type, SearchResultType::Unknown));
    }

    #[test]
    fn test_radar_address_to_search_result_no_confidence() {
        let mut addr = sample_address();
        addr.confidence = None;
        let result: SearchResult = addr.into();

        assert!(result.confidence.is_none());
        assert!(matches!(result.result_type, SearchResultType::Unknown));
    }

    #[test]
    fn test_radar_address_empty_fields() {
        let addr = RadarAddress {
            latitude: 0.0,
            longitude: 0.0,
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
        let result: SearchResult = addr.into();

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
}
