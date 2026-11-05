# everymap-providers-valhalla

[Valhalla](https://valhalla.github.io/valhalla/) routing provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs). Valhalla is keyless and self-hosted, so the provider needs only a deployment address.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Routing | `Router` | `ValhallaRouter`, ordered waypoints |

Not served: geocoding, place search, type-ahead, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

## Installation

```bash
cargo add everymap-providers-valhalla everymap-core
```

## Quick start

```rust
use everymap_core::domains::routing::{RouteOptions, Router};
use everymap_core::types::Coordinate;
use everymap_providers_valhalla::{ValhallaProviderConfiguration, ValhallaRouter};

#[tokio::main]
async fn main() {
    let router = ValhallaRouter::new(ValhallaProviderConfiguration {
        base_url: "http://localhost:8002".to_string(),
        user_agent: "my-map-application/1.0".to_string(),
    });
    let waypoints = [
        Coordinate::new(38.5816, -121.4944).unwrap(),
        Coordinate::new(38.5816, -121.4900).unwrap(),
    ];
    let response = router
        .calculate_route_with_waypoints(&waypoints, &RouteOptions::default())
        .await
        .unwrap();
    println!("{} meters", response.routes[0].distance);
}
```

## Notes

Requests are POSTs to `/route` with the service's locations, costing and directions options. Responses carry legs, the full maneuver table, authored display and speech text, and a precision-6 polyline decoded into the route shape.
