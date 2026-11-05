# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-tomtom

[TomTom](https://www.tomtom.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, traffic, isolines, map matching, tour optimization, raster tiles, and static maps.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `TomTomGeocoder` |
| Routing | `Router` | `TomTomRouter` ¹ |
| Traffic | `TrafficProvider` | `TomTomTraffic` |
| Raster tiles | `RasterTileProvider` | `TomTomTileProvider` |
| Isoline | `IsolineProvider` | `TomTomIsoline` |
| Matching | `RouteMatcher` | `TomTomRouteMatcher` |
| Tour | `TourPlanner` | `TomTomTourPlanner` |
| Imaging | `MapImageProvider` | `TomTomMapImageProvider` |

Not served: place search, type-ahead, distance matrix, vector tiles, elevation, positioning, attributes and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

¹ Routes two endpoints. A request with more waypoints is refused with `EveryMapError::UnsupportedDomain` rather than silently dropping the stops.

## Installation

```bash
cargo add everymap-providers-tomtom everymap-core
```

## Quick start

```rust
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_tomtom::client::TomTomClient;
use everymap_providers_tomtom::domain::search::TomTomGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // TomTom authenticates via the `key` query parameter.
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "key".to_string()));
    let client = Arc::new(TomTomClient::new(auth));

    let geocoder = TomTomGeocoder::new(client);
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

## Provider-specific features

Extension traits: `TomTomGeocoderExt` (reverse geocoding with rich TomTom result types) and `TomTomTrafficExt` (flow segment data and incident details).

Provider quirks are handled internally so core types just work: reverse geocoding returns `addresses`, isolines use `distanceBudgetInMeters`/`timeBudgetInSec` budgets, match-route uses the `/snapToRoads/1/snap` endpoint with longitude-first coordinates, tour optimization uses the Waypoint Optimization API (`optimizedOrder`), and core `AvoidType` values map to TomTom's `avoid*` parameters.