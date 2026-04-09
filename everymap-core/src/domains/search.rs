use async_trait::async_trait;
use crate::types::Coordinate;
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

/// Simplified search result returned by the core trait.
/// Provider implementations return richer types with a `From` conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub coordinate: Coordinate,
    pub address: String,
    pub title: Option<String>,
    pub result_type: Option<String>,
    pub id: Option<String>,
    pub distance: Option<f64>,
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