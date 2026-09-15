# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-mapbox

[MapBox](https://www.mapbox.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, isochrones, map matching, tour optimization, tiles, and static maps.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Search | `Geocoder` | ✅ |
| Routing | `Router` | ✅ |
| Isoline | `IsolineProvider` | ✅ |
| Matching | `RouteMatcher` | ✅ |
| Tour | `TourPlanner` | ✅ |
| Traffic | `TrafficProvider` | — |
| Tiling | `TileProvider` | ✅ |
| Positioning | `NetworkPositioner` | — |
| Attributes | `AttributeProvider` | — |
| Imaging | `MapImageProvider` | ✅ |

Unsupported domains (`MapBoxTraffic`, `MapBoxPositioner`, `MapBoxAttributeProvider`) return a clear `UnsupportedDomain` error.

## Installation

```bash
cargo add everymap-providers-mapbox everymap-core
```

## Quick start

```rust
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_mapbox::client::MapBoxClient;
use everymap_providers_mapbox::domain::search::MapBoxGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // MapBox authenticates via the `access_token` query parameter.
    let auth = Arc::new(ApiKeyProvider::new("YOUR_TOKEN".to_string(), "access_token".to_string()));
    let client = Arc::new(MapBoxClient::new(auth));

    let geocoder = MapBoxGeocoder::new(client);
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

## Provider-specific features

Extension traits: `MapBoxGeocoderExt` (Search Box v6 with `properties`-based results: `full_address`, `name`, `coordinates`, `bbox`, `context`) and `MapBoxRouterExt`.

The tour planner selects the Optimization API profile (`driving`/`walking`/`cycling`) from `TourOptions::transport_mode`. Static images use the default `streets-v12` style with no file extension in the URL, and vector tiles use MapBox's own tile scheme.