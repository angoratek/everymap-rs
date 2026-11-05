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
