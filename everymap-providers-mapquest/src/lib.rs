//! MapQuest provider for EveryMap.
//!
//! The crate implements the vendor-neutral EveryMap contracts against the
//! MapQuest web services platform
//! (`https://developer.mapquest.com/documentation/`):
//!
//! | Contract | Service |
//! |----------|---------|
//! | [`Geocoder`](everymap_core::domains::search::Geocoder) | Geocoding API v1 address and reverse endpoints |
//! | [`Router`](everymap_core::domains::routing::Router) | Directions API v2 route endpoint |
//! | [`DistanceMatrixProvider`](everymap_core::domains::matrix::DistanceMatrixProvider) | Directions API v2 route matrix endpoint |
//! | [`AutocompleteProvider`](everymap_core::domains::autocomplete::AutocompleteProvider) | Search Ahead API v3 prediction endpoint |
//! | [`TrafficProvider`](everymap_core::domains::traffic::TrafficProvider) | Traffic API v2 JSON flow and incident endpoints |
//! | [`RasterTileProvider`](everymap_core::domains::tiling::RasterTileProvider) | Tiles API rendered raster tiles |
//!
//! All services share one [`MapQuestClient`], which authenticates every request
//! with the consumer key as the documented `key` query parameter and applies
//! the workspace request pipeline: timeouts, bounded retries for idempotent
//! requests, optional rate limiting, verbose diagnostics and request
//! observation. The client never writes the key into an error message or a
//! log line; the shared pipeline redacts credential-bearing query values
//! before it reports a request.
//!
//! # Configuration
//!
//! [`MapQuestProviderConfiguration`] carries the platform base URL, the
//! consumer key and the `User-Agent` the service sees. The default targets the
//! public MapQuest platform.
//!
//! ```no_run
//! use std::sync::Arc;
//! use everymap_providers_mapquest::{
//!     MapQuestClient, MapQuestGeocoder, MapQuestProviderConfiguration,
//! };
//!
//! let client = Arc::new(MapQuestClient::new(MapQuestProviderConfiguration {
//!     base_url: "https://www.mapquestapi.com".to_string(),
//!     api_key: "your-consumer-key".to_string(),
//!     user_agent: "my-map-application/1.0".to_string(),
//! }));
//! let geocoder = MapQuestGeocoder::new(client);
//! # let _ = geocoder;
//! ```
//!
//! # Requests and responses
//!
//! Each provider sends the documented query parameters and reads the
//! `info` envelope when the service includes one. A non-zero
//! `info.statuscode` becomes
//! [`EveryMapError::ProviderError`](everymap_core::error::EveryMapError::ProviderError)
//! carrying the service status code and its messages. Non-success HTTP
//! statuses become the shared client's
//! [`EveryMapError::HttpError`](everymap_core::error::EveryMapError::HttpError)
//! or
//! [`EveryMapError::RateLimited`](everymap_core::error::EveryMapError::RateLimited).
//!
//! # Targets
//!
//! The crate builds for `wasm32-unknown-unknown`; no request path names a
//! `reqwest` type and the transport comes from `everymap-core`'s
//! target-specific default. Tests inject a transport and never open a socket.

pub mod autocomplete;
pub mod client;
mod error_mapping;
pub mod geocoding;
mod json_coordinates;
pub mod matrix;
pub mod place_search;
pub mod routing;
pub mod tiles;
pub mod traffic;

pub use autocomplete::MapQuestAutocompleteProvider;
pub use client::{
    MapQuestClient, MapQuestProviderConfiguration, DEFAULT_MAPQUEST_BASE_URL,
    DEFAULT_MAPQUEST_USER_AGENT, MAPQUEST_API_KEY_QUERY_PARAMETER, MAPQUEST_PROVIDER_NAME,
};
pub use geocoding::MapQuestGeocoder;
pub use matrix::MapQuestDistanceMatrix;
pub use place_search::MapQuestPlaceSearchProvider;
pub use routing::MapQuestRouter;
pub use tiles::{MapQuestRasterTileProvider, DEFAULT_MAPQUEST_TILE_STYLE};
pub use traffic::MapQuestTrafficProvider;
