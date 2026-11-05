# everymap-providers-osrm

[OSRM](https://project-osrm.org/) routing provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs). OSRM is keyless and self-hosted, so the provider needs only a deployment address.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Routing | `Router` | `OsrmRouter`, ordered waypoints |

Not served: geocoding, place search, type-ahead, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

## Installation

```bash
cargo add everymap-providers-osrm everymap-core
```

## Quick start

```rust
use everymap_core::domains::routing::{RouteOptions, Router};
use everymap_core::types::Coordinate;
use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};

#[tokio::main]
async fn main() {
    let router = OsrmRouter::new(OsrmProviderConfiguration {
        base_url: "http://localhost:5000".to_string(),
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

Routes carry one leg per consecutive waypoint pair, per-step maneuvers classified from the service's maneuver table, authored display and speech text, and the GeoJSON shape. The service has no shortest-path switch and cannot express a departure time, so `route_capabilities` reports the modes it can honour and the unsupported options are documented rather than approximated.
