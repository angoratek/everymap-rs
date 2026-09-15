# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-google

[Google Maps](https://mapsplatform.google.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, map matching, positioning, attributes, and static maps.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Search | `Geocoder` | ✅ |
| Routing | `Router` | ✅ |
| Isoline | `IsolineProvider` | — |
| Matching | `RouteMatcher` | ✅ |
| Tour | `TourPlanner` | — |
| Traffic | `TrafficProvider` | — |
| Tiling | `TileProvider` | — |
| Positioning | `NetworkPositioner` | ✅ |
| Attributes | `AttributeProvider` | ✅ |
| Imaging | `MapImageProvider` | ✅ |

Unsupported domains (`GoogleIsoline`, `GoogleTraffic`, `GoogleTourPlanner`, `GoogleTileProvider`) return a clear `UnsupportedDomain` error.

## Installation

```bash
cargo add everymap-providers-google everymap-core
```

## Quick start

```rust
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::domain::search::GoogleGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Google authenticates via the `key` query parameter.
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "key".to_string()));
    let client = Arc::new(GoogleClient::new(auth));

    let geocoder = GoogleGeocoder::new(client);
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

## Provider-specific features

Extension traits: `GoogleAttributeExt` (Roads API speed limits) and `GooglePositionerExt` (Geolocation API with cell-tower and Wi-Fi inputs).

Routing uses the legacy Directions API (the recommended Routes API v2 upgrade is on the roadmap). Unsupported option fields are never silently dropped — unsupported combinations emit `log::warn!` or are handled post-response (e.g. geocode `limit`).