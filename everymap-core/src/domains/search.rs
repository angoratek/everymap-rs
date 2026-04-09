use async_trait::async_trait;
use crate::types::{Coordinate, BoundingBox, Address};
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for forward geocoding (address → coordinate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeRequest<O> {
    pub query: String,
    pub options: O,
}

/// Request for reverse geocoding (coordinate → address).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseGeocodeRequest<O> {
    pub coordinate: Coordinate,
    pub options: O,
}

/// Request for discovering places/POIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverRequest<O> {
    pub query: String,
    pub options: O,
}

/// Request for autosuggest (type-ahead).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosuggestRequest<O> {
    pub query: String,
    pub options: O,
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
/// This trait covers the full search domain:
/// - Forward geocoding (address → coordinates)
/// - Reverse geocoding (coordinates → address)
/// - Discovery (POI search)
/// - Autosuggest (type-ahead suggestions)
///
/// Each provider implements these methods with its own Options type
/// and can return provider-specific response data via the associated Response type.
#[async_trait]
pub trait Geocoder: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn geocode(&self, req: GeocodeRequest<Self::Options>) -> EveryMapResult<Self::Response>;
    async fn reverse_geocode(&self, req: ReverseGeocodeRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}