# everymap-providers-radar

[Radar](https://radar.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — geocoding, routing, route matching, and route optimization.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Search | `Geocoder` | ✅ |
| Routing | `Router` | ✅ |
| Isoline | `IsolineProvider` | — |
| Matching | `RouteMatcher` | ✅ |
| Tour | `TourPlanner` | ✅ |
| Traffic | `TrafficProvider` | — |
| Tiling | `TileProvider` | — |
| Positioning | `NetworkPositioner` | — |
| Attributes | `AttributeProvider` | — |
| Imaging | `MapImageProvider` | — |

Unsupported domains (`RadarIsoline`, `RadarTraffic`, `RadarTileProvider`, `RadarPositioner`, `RadarAttributeProvider`, `RadarMapImageProvider`) return a clear `UnsupportedDomain` error.

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