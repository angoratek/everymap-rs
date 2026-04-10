use async_trait::async_trait;
use crate::types::{Coordinate, BoundingBox, Address};
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for forward geocoding (address → coordinate).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeocodeOptions {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Preferred response language (BCP 47 language tag, e.g., "en-US")
    pub language: Option<String>,
    /// Restrict results to these country codes (ISO 3166-1 alpha-3)
    pub country_codes: Vec<String>,
    /// Restrict search to this bounding box
    pub bounding_box: Option<BoundingBox>,
    /// Provider-specific options (HERE: political_view, show, qq; Google: region, components)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Options for reverse geocoding (coordinate → address).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReverseGeocodeOptions {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Search radius in meters
    pub radius: Option<f64>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Classification of a search result's match quality.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SearchResultType {
    /// Exact match (address matched precisely)
    ExactMatch,
    /// Approximate match (close but not exact)
    Approximate,
    /// Interpolated match (estimated from surrounding data)
    Interpolated,
    /// Unknown or provider-specific type
    Unknown,
}

/// A unified search result returned by the core trait.
///
/// Contains structured data that all providers can populate, plus an optional
/// `raw` field carrying provider-specific JSON for power users who need more detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier for this result (provider-specific)
    pub id: Option<String>,
    /// Geographic position of the result
    pub coordinate: Coordinate,
    /// Structured address components
    pub address: Address,
    /// Display title (e.g., "Brandenburg Gate")
    pub title: Option<String>,
    /// Classification of match quality
    pub result_type: SearchResultType,
    /// Distance from the search origin in meters (if applicable)
    pub distance: Option<f64>,
    /// Confidence score (0.0-1.0, higher = more confident)
    pub confidence: Option<f64>,
    /// POI categories (e.g., "restaurant", "hotel")
    pub categories: Vec<String>,
    /// Bounding box for the result (if available)
    pub bounding_box: Option<BoundingBox>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
}

/// Core trait for geocoding and search providers.
///
/// This trait covers the portable search domain:
/// - Forward geocoding (address → coordinates)
/// - Reverse geocoding (coordinates → address)
///
/// Provider-specific capabilities (e.g., HERE's `discover` and `autosuggest`)
/// are available through extension traits in provider crates.
#[async_trait]
pub trait Geocoder: Send + Sync {
    async fn geocode(&self, query: &str, options: &GeocodeOptions) -> EveryMapResult<SearchResponse>;
    async fn reverse_geocode(&self, coordinate: &Coordinate, options: &ReverseGeocodeOptions) -> EveryMapResult<SearchResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geocode_options_default() {
        let opts = GeocodeOptions::default();
        assert!(opts.limit.is_none());
        assert!(opts.language.is_none());
        assert!(opts.country_codes.is_empty());
        assert!(opts.bounding_box.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_geocode_options_with_fields() {
        let opts = GeocodeOptions {
            limit: Some(10),
            language: Some("en-US".to_string()),
            country_codes: vec!["DEU".to_string(), "FRA".to_string()],
            provider_extra: Some(serde_json::json!({"political_view": "ARG"})),
            ..Default::default()
        };
        assert_eq!(opts.limit, Some(10));
        assert_eq!(opts.language.as_deref(), Some("en-US"));
        assert_eq!(opts.country_codes.len(), 2);
        assert!(opts.provider_extra.is_some());
    }

    #[test]
    fn test_geocode_options_provider_extra_json() {
        let opts = GeocodeOptions {
            provider_extra: Some(serde_json::json!({
                "show": ["streetInfo", "mapReference"],
                "qq": "city=Berlin"
            })),
            ..Default::default()
        };
        let extra = opts.provider_extra.unwrap();
        assert!(extra.get("show").is_some());
        assert_eq!(extra["qq"], "city=Berlin");
    }

    #[test]
    fn test_reverse_geocode_options_default() {
        let opts = ReverseGeocodeOptions::default();
        assert!(opts.limit.is_none());
        assert!(opts.language.is_none());
        assert!(opts.radius.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_search_result_type_serialization() {
        assert_eq!(
            serde_json::to_string(&SearchResultType::ExactMatch).unwrap(),
            "\"ExactMatch\""
        );
        assert_eq!(
            serde_json::to_string(&SearchResultType::Unknown).unwrap(),
            "\"Unknown\""
        );
        let rt: SearchResultType = serde_json::from_str("\"Approximate\"").unwrap();
        assert_eq!(rt, SearchResultType::Approximate);
    }

    #[test]
    fn test_search_result_construction() {
        let result = SearchResult {
            id: Some("test-id".to_string()),
            coordinate: Coordinate::new(52.5, 13.4).unwrap(),
            address: Address::empty(),
            title: Some("Test Place".to_string()),
            result_type: SearchResultType::ExactMatch,
            distance: Some(150.0),
            confidence: Some(0.95),
            categories: vec!["restaurant".to_string()],
            bounding_box: None,
            raw: Some(serde_json::json!({"provider_specific": true})),
        };
        assert_eq!(result.id.as_deref(), Some("test-id"));
        assert_eq!(result.coordinate.lat, 52.5);
        assert_eq!(result.result_type, SearchResultType::ExactMatch);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_search_response_serialization_roundtrip() {
        let response = SearchResponse {
            items: vec![SearchResult {
                id: Some("abc".to_string()),
                coordinate: Coordinate::new(1.0, 2.0).unwrap(),
                address: Address::empty(),
                title: None,
                result_type: SearchResultType::Unknown,
                distance: None,
                confidence: None,
                categories: vec![],
                bounding_box: None,
                raw: None,
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].id.as_deref(), Some("abc"));
    }
}