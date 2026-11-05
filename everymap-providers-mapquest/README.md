# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-mapquest

[MapQuest](https://developer.mapquest.com/) provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs).

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `MapQuestGeocoder` |
| Place search | `PlaceSearchProvider` | `MapQuestPlaceSearchProvider` |
| Type-ahead | `AutocompleteProvider` | `MapQuestAutocompleteProvider` |
| Routing | `Router` | `MapQuestRouter`, ordered waypoints |
| Distance matrix | `DistanceMatrixProvider` | `MapQuestDistanceMatrix` |
| Traffic | `TrafficProvider` | `MapQuestTrafficProvider` |
| Raster tiles | `RasterTileProvider` | `MapQuestRasterTileProvider` |

Not served: vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

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
