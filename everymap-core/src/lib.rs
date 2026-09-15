//! Core traits, types, and error handling for the EveryMap geospatial API abstraction.
//!
//! This crate defines 10 domain traits (geocoding, routing, isolines, map matching,
//! tour planning, traffic, tiling, positioning, attributes, and imaging) that provider
//! crates implement. It also provides shared infrastructure: authentication, HTTP client
//! logic, error types, and unsupported-domain stub macros.
//!
//! # Architecture
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`domains`] | 10 domain traits with concrete option/response types |
//! | [`auth`] | `AuthProvider`, `ApiKeyProvider`, `HeaderAuthProvider`, `OAuth2Provider` |
//! | [`client`] | Shared `ProviderClient` HTTP logic |
//! | [`types`] | `Coordinate`, `BoundingBox`, `Address`, `Polyline` |
//! | [`error`] | Structured `EveryMapError` with 9 variants |
//!
//! # Quick Start
//!
//! ```no_run
//! use everymap_core::domains::search::{Geocoder, GeocodeOptions};
//! use everymap_core::auth::ApiKeyProvider;
//! use std::sync::Arc;
//! # async fn example() {
//! let auth = Arc::new(ApiKeyProvider::new("key".into(), "apiKey".into()));
//! // Provider crates implement the domain traits:
//! // use everymap_providers_here::domain::search::HereGeocoder;
//! # }
//! ```

pub mod auth;
pub mod client;
pub mod domains;
pub mod error;
pub mod types;
pub mod unsupported;

pub use auth::*;
pub use domains::*;
pub use error::*;
pub use types::*;
