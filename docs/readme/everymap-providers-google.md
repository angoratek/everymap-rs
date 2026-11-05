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

Routing uses the legacy Directions API (the recommended Routes API v2 upgrade is on the roadmap). Unsupported option fields are never silently dropped — unsupported combinations emit `log::warn!` or are handled post-response (e.g. geocode `limit`).