use crate::error::EveryMapResult;
use crate::types::{Address, BoundingBox, Coordinate};
use async_trait::async_trait;
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
    async fn geocode(
        &self,
        query: &str,
        options: &GeocodeOptions,
    ) -> EveryMapResult<SearchResponse>;
    async fn reverse_geocode(
        &self,
        coordinate: &Coordinate,
        options: &ReverseGeocodeOptions,
    ) -> EveryMapResult<SearchResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geocode_options_default() {
        let options = GeocodeOptions::default();
        assert!(options.limit.is_none());
        assert!(options.language.is_none());
        assert!(options.country_codes.is_empty());
        assert!(options.bounding_box.is_none());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_geocode_options_with_fields() {
        let options = GeocodeOptions {
            limit: Some(10),
            language: Some("en-US".to_string()),
            country_codes: vec!["DEU".to_string(), "FRA".to_string()],
            provider_extra: Some(serde_json::json!({"political_view": "ARG"})),
            ..Default::default()
        };
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.language.as_deref(), Some("en-US"));
        assert_eq!(options.country_codes.len(), 2);
        assert!(options.provider_extra.is_some());
    }

    #[test]
    fn test_geocode_options_provider_extra_json() {
        let options = GeocodeOptions {
            provider_extra: Some(serde_json::json!({
                "show": ["streetInfo", "mapReference"],
                "qq": "city=Berlin"
            })),
            ..Default::default()
        };
        let extra = options.provider_extra.unwrap();
        assert!(extra.get("show").is_some());
        assert_eq!(extra["qq"], "city=Berlin");
    }

    #[test]
    fn test_reverse_geocode_options_default() {
        let options = ReverseGeocodeOptions::default();
        assert!(options.limit.is_none());
        assert!(options.language.is_none());
        assert!(options.radius.is_none());
        assert!(options.provider_extra.is_none());
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

    // --- SearchResultType all variants serde roundtrip ---

    #[test]
    fn test_search_result_type_all_variants_serde() {
        let variants = [
            SearchResultType::ExactMatch,
            SearchResultType::Approximate,
            SearchResultType::Interpolated,
            SearchResultType::Unknown,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: SearchResultType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_search_result_type_all_variants_distinct() {
        let variants = [
            SearchResultType::ExactMatch,
            SearchResultType::Approximate,
            SearchResultType::Interpolated,
            SearchResultType::Unknown,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- SearchResponse serde roundtrip (empty) ---

    #[test]
    fn test_search_response_empty_serde_roundtrip() {
        let response = SearchResponse { items: vec![] };
        let json = serde_json::to_string(&response).unwrap();
        let back: SearchResponse = serde_json::from_str(&json).unwrap();
        assert!(back.items.is_empty());
    }

    // --- SearchResponse serde roundtrip with full fields ---

    #[test]
    fn test_search_response_full_serde_roundtrip() {
        let ne = crate::types::BoundingBox::new(
            Coordinate::new(52.6, 13.5).unwrap(),
            Coordinate::new(52.4, 13.3).unwrap(),
        );
        let response = SearchResponse {
            items: vec![SearchResult {
                id: Some("id-123".to_string()),
                coordinate: Coordinate::new(52.52, 13.40).unwrap(),
                address: Address::from_label("Brandenburg Gate".to_string()),
                title: Some("Brandenburg Gate".to_string()),
                result_type: SearchResultType::ExactMatch,
                distance: Some(250.0),
                confidence: Some(0.98),
                categories: vec!["monument".to_string(), "landmark".to_string()],
                bounding_box: Some(ne),
                raw: Some(serde_json::json!({"source": "here"})),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: SearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].id.as_deref(), Some("id-123"));
        assert_eq!(back.items[0].title.as_deref(), Some("Brandenburg Gate"));
        assert_eq!(back.items[0].result_type, SearchResultType::ExactMatch);
        assert_eq!(back.items[0].distance, Some(250.0));
        assert_eq!(back.items[0].confidence, Some(0.98));
        assert_eq!(back.items[0].categories.len(), 2);
        assert!(back.items[0].bounding_box.is_some());
        assert!(back.items[0].raw.is_some());
    }

    // --- GeocodeOptions with provider_extra serde roundtrip ---

    #[test]
    fn test_geocode_options_serde_roundtrip() {
        let options = GeocodeOptions {
            limit: Some(5),
            language: Some("de-DE".to_string()),
            country_codes: vec!["DEU".to_string()],
            bounding_box: Some(crate::types::BoundingBox::new(
                Coordinate::new(52.6, 13.5).unwrap(),
                Coordinate::new(52.4, 13.3).unwrap(),
            )),
            provider_extra: Some(serde_json::json!({"political_view": "ARG"})),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: GeocodeOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.limit, Some(5));
        assert_eq!(back.language.as_deref(), Some("de-DE"));
        assert_eq!(back.country_codes.len(), 1);
        assert!(back.bounding_box.is_some());
        assert!(back.provider_extra.is_some());
    }

    // --- ReverseGeocodeOptions serde roundtrip ---

    #[test]
    fn test_reverse_geocode_options_serde_roundtrip() {
        let options = ReverseGeocodeOptions {
            limit: Some(1),
            language: Some("fr".to_string()),
            radius: Some(500.0),
            provider_extra: Some(serde_json::json!({"include_shapes": true})),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: ReverseGeocodeOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.limit, Some(1));
        assert_eq!(back.radius, Some(500.0));
        assert!(back.provider_extra.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_search_result_zero_distance() {
        let result = SearchResult {
            id: None,
            coordinate: Coordinate::ORIGIN,
            address: Address::empty(),
            title: None,
            result_type: SearchResultType::Unknown,
            distance: Some(0.0),
            confidence: Some(0.0),
            categories: vec![],
            bounding_box: None,
            raw: None,
        };
        assert_eq!(result.distance, Some(0.0));
        assert_eq!(result.confidence, Some(0.0));
    }

    #[test]
    fn test_search_result_large_coordinates() {
        let result = SearchResult {
            id: None,
            coordinate: Coordinate::new(89.9999, 179.9999).unwrap(),
            address: Address::empty(),
            title: None,
            result_type: SearchResultType::Unknown,
            distance: None,
            confidence: None,
            categories: vec![],
            bounding_box: None,
            raw: None,
        };
        assert!((result.coordinate.lat - 89.9999).abs() < f64::EPSILON);
    }

    #[test]
    fn test_search_result_with_empty_categories() {
        let result = SearchResult {
            id: Some("x".to_string()),
            coordinate: Coordinate::ORIGIN,
            address: Address::empty(),
            title: None,
            result_type: SearchResultType::Approximate,
            distance: None,
            confidence: None,
            categories: vec![],
            bounding_box: None,
            raw: None,
        };
        assert!(result.categories.is_empty());
    }
}
