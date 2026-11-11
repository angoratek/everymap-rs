# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-google

[Google Maps](https://mapsplatform.google.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, map matching, positioning, attributes, and static maps.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `GoogleGeocoder` |
| Routing | `Router` | `GoogleRouter` ¹ |
| Matching | `RouteMatcher` | `GoogleRouteMatcher` |
| Positioning | `NetworkPositioner` | `GooglePositioner` |
| Attributes | `AttributeProvider` | `GoogleAttributeProvider` |
| Imaging | `MapImageProvider` | `GoogleMapImageProvider` |

Not served: place search, type-ahead, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, tour and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

¹ Routes two endpoints. A request with more waypoints is refused with `EveryMapError::UnsupportedDomain` rather than silently dropping the stops.

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

Routing uses the Routes API v2 (`computeRoutes`). Each domain derives its endpoint from one configurable service root (`with_base_url`), defaulting to the vendor's public host for that API.

Google's in-body `status` maps `REQUEST_DENIED` to `AuthError`, `OVER_QUERY_LIMIT` and `OVER_DAILY_LIMIT` to `RateLimited`, and every other non-success status to `ProviderError` carrying the status verbatim. A payload that decodes as JSON but lacks a required field is `ProviderError` with code `INVALID_RESPONSE`. `raw` is populated on geocode results and left unset on routing, matching and positioning results. Unsupported option fields are never silently dropped — unsupported combinations emit `log::warn!` or are handled post-response (e.g. geocode `limit`).