//! OSRM routing provider for EveryMap.
//!
//! [OSRM](https://project-osrm.org/) is a self-hosted, keyless routing engine.
//! This crate implements [`everymap_core::domains::routing::Router`] against
//! its HTTP route service: ordered waypoints and vendor-neutral
//! [`everymap_core::domains::routing::RouteOptions`] go in, and a
//! navigation-grade [`everymap_core::domains::routing::RouteResult`] with
//! legs, turn-by-turn steps, authored instructions and a GeoJSON shape comes
//! out.
//!
//! # Configuration
//!
//! [`OsrmProviderConfiguration`] carries the deployment base URL and the
//! `User-Agent` the server sees. The default targets a local OSRM
//! installation listening on port 5000.
//!
//! ```no_run
//! use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};
//!
//! let router = OsrmRouter::new(OsrmProviderConfiguration::default());
//! # let _ = router;
//! ```
//!
//! # Request and response
//!
//! Requests are GETs to
//! `{base_url}/route/v1/{profile}/{longitude},{latitude};…` with
//! `overview=full`, `geometries=geojson`, `steps=true` and an `exclude` value
//! when features are avoided. The first route is read; its GeoJSON geometry
//! becomes the route shape, and every leg carries its own steps and geometry.
//!
//! # Failure channels
//!
//! A malformed success body is
//! [`everymap_core::error::EveryMapError::SerializationError`]. An OSRM
//! `code` other than `Ok`, such as `NoRoute` or `NoSegment`, is
//! [`everymap_core::error::EveryMapError::ProviderError`] carrying the service
//! code and message. A non-success status without an OSRM envelope is
//! [`everymap_core::error::EveryMapError::HttpError`]; HTTP 429 is reported as
//! [`everymap_core::error::EveryMapError::RateLimited`] by the shared client.

mod client;
mod error_mapping;
pub mod route;

pub use client::{
    OsrmProviderConfiguration, DEFAULT_OSRM_BASE_URL, DEFAULT_OSRM_USER_AGENT, OSRM_PROVIDER_NAME,
};
pub use route::OsrmRouter;
