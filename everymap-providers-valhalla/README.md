# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-valhalla

[Valhalla](https://valhalla.github.io/valhalla/) routing provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs). Valhalla is keyless and self-hosted, so the provider needs only a deployment address.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Routing | `Router` | ✅ |

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
