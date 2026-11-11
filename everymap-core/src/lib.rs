//! Core traits, types, and error handling for the EveryMap geospatial API abstraction.
//!
//! This crate defines the vendor-neutral domain contracts provider crates implement:
//! the original 10 domain traits (geocoding, routing, isolines, map matching, tour
//! planning, traffic, tiling, positioning, attributes, and imaging) plus the
//! navigation-grade routing surface, the typed raster and vector tile surfaces, and
//! the autocomplete, distance-matrix, elevation and offline-routing traits.
//! It also provides shared infrastructure: authentication, HTTP client logic, error
//! types, and unsupported-domain stub macros.
//!
//! # Architecture
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`domains`] | Domain traits with concrete option/response types, including navigation-grade routing and typed tiles |
//! | [`auth`] | `AuthProvider`, `ApiKeyProvider`, `HeaderAuthProvider`, `OAuth2Provider` |
//! | [`client`] | Shared `ProviderClient` HTTP logic, rate limiting, retries, observers |
//! | [`http`] | Backend-neutral `HttpRequest`, `HttpResponse`, `HttpTransport` |
//! | [`tile_source`] | Shared `{z}/{x}/{y}` template, pacing and tile-coordinate validation |
//! | [`types`] | `Coordinate`, `BoundingBox`, `Address`, `Polyline` |
//! | [`error`] | Structured `EveryMapError` with 10 variants |
//!
//! The crate builds for `wasm32-unknown-unknown`; on that target the default
//! transport is `window.fetch` and `reqwest` is not compiled.
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
pub mod http;
pub mod tile_source;
pub mod types;
pub mod unsupported;

pub use auth::*;
pub use domains::*;
pub use error::*;
pub use tile_source::*;
pub use types::*;
