# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-mapquest

[MapQuest](https://developer.mapquest.com/) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs).

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Search | `Geocoder` | ✅ |
| Settled place search | `PlaceSearchProvider` | ✅ |
| Autocomplete | `AutocompleteProvider` | ✅ |
| Routing | `Router` | ✅ |
| Distance matrix | `DistanceMatrixProvider` | ✅ |
| Traffic | `TrafficProvider` | ✅ |
| Raster tiles | `RasterTileProvider` | ✅ |

## Installation

```bash
cargo add everymap-providers-mapquest everymap-core
```

## Quick start

```rust
use everymap_providers_mapquest::{
    MapQuestClient, MapQuestGeocoder, MapQuestProviderConfiguration,
};
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let client = Arc::new(MapQuestClient::new(MapQuestProviderConfiguration {
        base_url: "https://www.mapquestapi.com".to_string(),
        api_key: "YOUR_CONSUMER_KEY".to_string(),
        user_agent: "my-map-application/1.0".to_string(),
    }));
    let geocoder = MapQuestGeocoder::new(client);
    let result = geocoder.geocode("Denver", &GeocodeOptions::default()).await.unwrap();
    for item in &result.items {
        println!("{}", item.title.as_deref().unwrap_or("?"));
    }
}
```

## Notes

Every service shares one client, which authenticates with the consumer key as the documented `key` query parameter and redacts that value from diagnostics. The router honours `RouteOptions::unit_system`, the traffic provider honours `TrafficOptions::bounding_box`, and the geocoder fills `SearchQuality` alongside the confidence classification.
