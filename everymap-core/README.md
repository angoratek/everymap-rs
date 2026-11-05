# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-core

Core traits, types, and error handling for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — the foundation every provider crate builds on.

## What's inside

- **16 contract traits** with concrete option/response types (no associated types, so `Box<dyn Trait>` runtime dispatch works): `Geocoder`, `PlaceSearchProvider`, `AutocompleteProvider`, `Router`, `DistanceMatrixProvider`, `TrafficProvider`, `RasterTileProvider`, `VectorTileProvider`, `ElevationProvider`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`, `OfflineRoutingProvider`
- **Navigation-grade routing**: `Router::calculate_route_with_waypoints` takes ordered waypoints and returns legs, typed maneuvers with stable labels, display and speech instructions, lane guidance, step and leg shapes, and `route_capabilities`
- **Typed tiles**: `RasterTileProvider` with a typed request and a response that echoes format and resolution; `VectorTileProvider` with a source descriptor carrying cache namespace, attribution, zoom span, wire encoding and style hints
- **Shared types**: `Coordinate`, `BoundingBox`, `Address`, `Polyline`
- **Auth**: `AuthProvider` trait with `ApiKeyProvider` and `HeaderAuthProvider` — API keys are zeroized on drop
- **HTTP**: backend-neutral `ProviderClient` on the `HttpTransport` seam — `reqwest` natively, `window.fetch` on WebAssembly, with token-bucket rate limiting, idempotent-GET retries, timeouts, and credential redaction
- **Errors**: structured `EveryMapError` (`HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, and more)
- **Stub macros**: 8 `unsupported_*!` macros so provider crates stub out unsupported contracts cleanly

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