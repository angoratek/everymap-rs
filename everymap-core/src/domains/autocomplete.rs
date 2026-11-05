use crate::error::EveryMapResult;
use crate::types::{BoundingBox, Coordinate};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for a type-ahead completion request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutocompleteOptions {
    /// Maximum number of candidates to return.
    pub limit: Option<u32>,
    /// Preferred response language (BCP 47 language tag, e.g., `en-US`).
    pub language: Option<String>,
    /// An area to prefer or restrict candidates to, typically the viewport.
    pub bounding_box: Option<BoundingBox>,
    /// Restrict candidates to these country codes (ISO 3166-1 alpha-3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub country_codes: Vec<String>,
    /// Provider-specific options for advanced use cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// One completion candidate for a partially typed query.
///
/// A candidate may describe a region or a street with no single point, so the
/// coordinate is optional; a caller that needs a point waits for the settled
/// search result instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteResult {
    /// Provider-stable identifier for the suggestion, when reported.
    pub id: Option<String>,
    /// The primary display line.
    pub title: String,
    /// A secondary line (region, country or address context), when reported.
    pub subtitle: Option<String>,
    /// The suggested position, when the provider reports one.
    pub coordinate: Option<Coordinate>,
    /// Bounding box for the suggestion, when reported.
    pub bounding_box: Option<BoundingBox>,
    /// The match granularity the provider classifies for the suggestion, when
    /// it reports one. A host that shows a quality for a settled result can
    /// show the same scale while the visitor is still typing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<crate::domains::search::SearchQuality>,
    /// POI categories or feature classes (e.g., `restaurant`, `city`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// Provider-specific raw data for advanced use cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// The completion candidates one query returns, in relevance order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteResponse {
    /// Candidates ordered most relevant first.
    pub items: Vec<AutocompleteResult>,
}

/// Type-ahead completion for a partially typed search query.
///
/// A provider that offers a dedicated completion endpoint implements this
/// trait; a caller that only has a forward geocoder can debounce its own
/// query instead. The trait is separate from [`crate::domains::search::Geocoder`]
/// because completion endpoints usually have their own rate limits, result
/// caps and billing.
#[async_trait]
pub trait AutocompleteProvider: Send + Sync {
    /// Returns completion candidates for a partial query.
    ///
    /// # Parameters
    /// - `query` — the text entered so far. A provider may require a minimum
    ///   length; a shorter query returns an empty response.
    /// - `options` — the result cap, language, preferred area and any
    ///   provider-specific options.
    ///
    /// # Returns
    /// Candidates in the provider's relevance order, at most
    /// [`AutocompleteOptions::limit`].
    ///
    /// # Errors
    /// Returns [`crate::error::EveryMapError::ValidationError`] when the query
    /// cannot be used, [`crate::error::EveryMapError::TransportError`] when the
    /// request cannot reach the provider, and
    /// [`crate::error::EveryMapError::SerializationError`] when the response
    /// cannot be decoded.
    async fn autocomplete(
        &self,
        query: &str,
        options: &AutocompleteOptions,
    ) -> EveryMapResult<AutocompleteResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_options_default() {
        let options = AutocompleteOptions::default();
        assert!(options.limit.is_none());
        assert!(options.language.is_none());
        assert!(options.bounding_box.is_none());
        assert!(options.country_codes.is_empty());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_autocomplete_result_roundtrip() {
        let response = AutocompleteResponse {
            items: vec![AutocompleteResult {
                id: Some("node/1".to_string()),
                title: "Sacramento".to_string(),
                subtitle: Some("California, United States".to_string()),
                coordinate: Some(Coordinate::new(38.5816, -121.4944).unwrap()),
                bounding_box: None,
                quality: Some(crate::domains::search::SearchQuality::City),
                categories: vec!["city".to_string()],
                raw: None,
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: AutocompleteResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].title, "Sacramento");
        assert_eq!(back.items[0].categories, vec!["city".to_string()]);
    }

    #[test]
    fn test_autocomplete_result_without_coordinate_roundtrips() {
        let result = AutocompleteResult {
            id: None,
            title: "J Street".to_string(),
            subtitle: None,
            coordinate: None,
            bounding_box: None,
            quality: None,
            categories: Vec::new(),
            raw: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: AutocompleteResult = serde_json::from_str(&json).unwrap();
        assert!(back.coordinate.is_none());
    }
}
