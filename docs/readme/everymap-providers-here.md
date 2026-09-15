# everymap-providers-here

[HERE Technologies](https://www.here.com) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — the most complete implementation, covering all 10 geospatial domains.

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
| Positioning | `NetworkPositioner` | ✅ |
| Attributes | `AttributeProvider` | ✅ |
| Imaging | `MapImageProvider` | ✅ |

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

Every domain trait is implemented, and each domain module carries HERE-specific rich types (e.g. `HereRouteOptions` with EV, fuel, truck, and driver parameters; `HereMatchedRoute` for Route Matching v8 with compound `fastest;car;traffic:disabled` modes).

Extension traits expose HERE-only capabilities beyond the core traits: `HereGeocoderExt` (rich Search API discovery, e.g. `discover()`), `HereTrafficExt`, `HerePositionerExt`, `HereTourPlannerExt`, and `HereAttributeExt`.

HERE uses its own tiling scheme for the `TileProvider` — see the [root README](https://github.com/angoratek/everymap-rs#cli-usage) for the Berlin z14 example coordinates.