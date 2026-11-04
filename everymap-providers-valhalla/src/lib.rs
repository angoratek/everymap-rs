//! Valhalla routing provider for EveryMap.
//!
//! [Valhalla](https://valhalla.github.io/valhalla/) is a self-hosted, keyless
//! routing engine. This crate implements
//! [`everymap_core::domains::routing::Router`] against its HTTP `/route`
//! endpoint: ordered waypoints and vendor-neutral
//! [`everymap_core::domains::routing::RouteOptions`] go in, and a
//! navigation-grade [`everymap_core::domains::routing::RouteResult`] with
//! legs, turn-by-turn steps and the decoded geometry comes out.
//!
//! # Configuration
//!
//! [`ValhallaProviderConfiguration`] carries the operator's server base URL
//! and the `User-Agent` the server sees. The default targets a local Valhalla
//! container listening on port 8002.
//!
//! ```no_run
//! use everymap_providers_valhalla::{ValhallaProviderConfiguration, ValhallaRouter};
//!
//! let router = ValhallaRouter::new(ValhallaProviderConfiguration::default());
//! # let _ = router;
//! ```
//!
//! # Request and response
//!
//! Requests are JSON POSTs to `{base_url}/route`. Every leg shape is a
//! precision-6 encoded polyline decoded inside the crate; maneuver `type`
//! integers map onto [`everymap_core::domains::routing::ManeuverType`] through
//! a total classification table. Distances are normalized to meters and
//! durations to seconds.
//!
//! # Failure channels
//!
//! A body that is not a usable Valhalla trip is
//! [`everymap_core::error::EveryMapError::SerializationError`]. A documented
//! Valhalla `error` refusal with a `4xx` status is
//! [`everymap_core::error::EveryMapError::ProviderError`] carrying the
//! service's message verbatim. A `5xx` response is
//! [`everymap_core::error::EveryMapError::HttpError`]; HTTP 429 is reported as
//! [`everymap_core::error::EveryMapError::RateLimited`] by the shared client.

mod client;
mod error_mapping;
mod maneuver_mapping;
mod polyline;
mod routing;

pub use client::{
    ValhallaProviderConfiguration, DEFAULT_VALHALLA_BASE_URL, DEFAULT_VALHALLA_USER_AGENT,
    VALHALLA_PROVIDER_NAME,
};
pub use routing::ValhallaRouter;
