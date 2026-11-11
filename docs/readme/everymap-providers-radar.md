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

Extension traits expose Radar-only capabilities beyond the core contracts: `RadarGeocoderExt` (autocomplete, IP geocoding, address validation), `RadarSearchExt` (place and chain search), `RadarRouterExt` (distance and matrix endpoints) and `RadarMatchingExt` (route matching with road attributes), plus the `RadarPlace` and `RadarChain` types. These surfaces are specific to Radar; the core place-search, type-ahead and distance-matrix contracts stay unimplemented.

Each domain derives its endpoint from one configurable service root (`with_base_url`), defaulting to `https://api.radar.io`.

Core `avoid` (tolls, highways and ferries) and `alternatives` are wired to the Directions API; a `provider_extra` value overrides either. The response envelope's `meta.code` maps 401 and 403 to `AuthError`, 429 to `RateLimited`, and every other non-200 code to `ProviderError` carrying the numeric code. A payload that decodes as JSON but lacks a required field is `ProviderError` with code `INVALID_RESPONSE`; a body that does not decode is a serialization error. `raw` is populated on address results and tour responses and left unset on routing and matching results. Field naming quirks are mapped internally (routing steps use `snake_case`, legs use `camelCase`).