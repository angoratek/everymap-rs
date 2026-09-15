# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-core

Core traits, types, and error handling for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — the foundation every provider crate builds on.

## What's inside

- **10 domain traits** with concrete option/response types (no associated types, so `Box<dyn Trait>` runtime dispatch works): `Geocoder`, `Router`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `TrafficProvider`, `TileProvider`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`
- **Shared types**: `Coordinate`, `BoundingBox`, `Address`, `Polyline`
- **Auth**: `AuthProvider` trait with `ApiKeyProvider` and `HeaderAuthProvider` — API keys are zeroized on drop
- **HTTP**: `ProviderClient` with shared request, JSON, and API-key redaction logic
- **Errors**: structured `EveryMapError` (`HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, and more)
- **Stub macros**: 7 `unsupported_*!` macros so provider crates stub out unsupported domains cleanly

Every options struct carries a `provider_extra: Option<serde_json::Value>` escape hatch for provider-specific parameters.

## Installation

```bash
cargo add everymap-core
```

## Quick start

Write provider-agnostic code against core traits — the provider implementation is injected at construction time:

```rust
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::types::Coordinate;
use std::sync::Arc;

// API keys are zeroized when the auth provider is dropped.
fn make_auth() -> Arc<ApiKeyProvider> {
    Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "apiKey".to_string()))
}

// Any provider crate's geocoder implements this trait:
async fn first_coordinate(geocoder: &dyn Geocoder, query: &str) -> Option<Coordinate> {
    let result = geocoder.geocode(query, &GeocodeOptions::default()).await.ok()?;
    result.items.into_iter().next().map(|item| item.coordinate)
}
```

Pair this crate with a provider implementation — see [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) and friends — or use [`everymap-cli`](https://crates.io/crates/everymap-cli) for command-line access.