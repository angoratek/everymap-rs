# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-tomtom

[TomTom](https://www.tomtom.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, traffic, isolines, map matching, tour optimization, tiles, and static maps.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Search | `Geocoder` | ✅ |
| Routing | `Router` | ✅ |
| Isoline | `IsolineProvider` | ✅ |
| Matching | `RouteMatcher` | ✅ |
| Tour | `TourPlanner` | ✅ |
| Traffic | `TrafficProvider` | ✅ |
| Tiling | `TileProvider` | ✅ |
| Positioning | `NetworkPositioner` | — |
| Attributes | `AttributeProvider` | — |
| Imaging | `MapImageProvider` | ✅ |

Unsupported domains (`TomTomPositioner`, `TomTomAttributeProvider`) return a clear `UnsupportedDomain` error.

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