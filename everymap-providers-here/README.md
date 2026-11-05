# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-here

[HERE Technologies](https://www.here.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — the most complete implementation, serving ten of the sixteen contracts.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `HereGeocoder` |
| Routing | `Router` | `HereRouter` ¹ |
| Traffic | `TrafficProvider` | `HereTraffic` |
| Vector tiles | `VectorTileProvider` | `HereTileProvider` |
| Isoline | `IsolineProvider` | `HereIsoline` |
| Matching | `RouteMatcher` | `HereRouteMatcher` |
| Tour | `TourPlanner` | `HereTourPlanner` |
| Positioning | `NetworkPositioner` | `HerePositioner` |
| Attributes | `AttributeProvider` | `HereAttributeProvider` |
| Imaging | `MapImageProvider` | `HereMapImageProvider` |

Not served: place search, type-ahead, distance matrix, raster tiles, elevation and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

¹ Routes two endpoints. A request with more waypoints is refused with `EveryMapError::UnsupportedDomain` rather than silently dropping the stops.

## Installation

```bash
cargo add everymap-providers-here everymap-core
```

## Quick start

```rust
use everymap_core::auth::ApiKeyProvider;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::search::HereGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // HERE authenticates via the `apiKey` query parameter.
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));

    let geocoder = HereGeocoder::new(client);
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

## Provider-specific features

Every contract HERE serves has an implementation, and each domain module carries HERE-specific rich types (e.g. `HereRouteOptions` with EV, fuel, truck, and driver parameters; `HereMatchedRoute` for Route Matching v8 with compound `fastest;car;traffic:disabled` modes).

Extension traits expose HERE-only capabilities beyond the core traits: `HereGeocoderExt` (rich Search API discovery, e.g. `discover()`), `HereTrafficExt`, `HerePositionerExt`, `HereTourPlannerExt`, and `HereAttributeExt`.

HERE serves MVT vector tiles through `VectorTileProvider`, requesting the Vector Tile API's `omv` output between zoom 0 and 17; another layer is one call away through `fetch_vector_tile_in_layer`. See the [root README](https://github.com/angoratek/everymap-rs#cli-usage) for the Berlin z14 example coordinates.