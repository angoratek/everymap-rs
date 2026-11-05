# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-radar

[Radar](https://radar.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, route matching, and route optimization.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `RadarGeocoder` |
| Routing | `Router` | `RadarRouter` ¹ |
| Matching | `RouteMatcher` | `RadarRouteMatcher` |
| Tour | `TourPlanner` | `RadarTourPlanner` |

Not served: place search, type-ahead, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

¹ Routes two endpoints. A request with more waypoints is refused with `EveryMapError::UnsupportedDomain` rather than silently dropping the stops.

## Installation

```bash
cargo add everymap-providers-radar everymap-core
```

## Quick start

```rust
use everymap_core::auth::HeaderAuthProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_radar::client::RadarClient;
use everymap_providers_radar::domain::search::RadarGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Radar authenticates via the Authorization header.
    let auth = Arc::new(HeaderAuthProvider::new("YOUR_SECRET_KEY".to_string()));
    let client = Arc::new(RadarClient::new(auth));

    let geocoder = RadarGeocoder::new(client);
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

## Provider-specific features

Extension traits expose Radar-only capabilities: `RadarSearchExt` (autocomplete, IP geocoding, address validation), `RadarGeocoderExt`, `RadarRouterExt` (distance matrix), and `RadarMatchingExt`, plus `RadarChain` and `RadarPlace` types for place/chain search.

Field naming quirks are mapped internally (routing steps use `snake_case`, legs use `camelCase`); core `avoid` and `alternatives` fields are currently only usable via the `provider_extra` escape hatch.