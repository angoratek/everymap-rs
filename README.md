# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs with provider abstraction.

## Overview

EveryMap-RS provides a unified interface for geospatial services across multiple providers. Currently implementing **HERE Technologies** APIs with plans for MapBox, TomTom, and Google Maps.

The architecture uses **domain-driven design** — each geospatial capability (routing, search, traffic, etc.) is defined as a trait in `everymap-core`, with provider-specific implementations in separate crates. This allows switching providers without changing business logic.

## Architecture

```
everymap-rs/
├── everymap-core/           # Core traits, types, auth, error
│   └── src/
│       ├── domains/         # 10 domain traits (Geocoder, Router, etc.)
│       ├── types/           # Coordinate, Polyline, FlexiblePolyline
│       ├── auth/            # AuthProvider, ApiKeyProvider
│       └── error/           # EveryMapError
└── everymap-providers-here/ # HERE Technologies implementation
    └── src/
        ├── client/          # HereClient (HTTP + auth)
        └── domain/          # 10 domain implementations
            ├── search/      # Geocode, Discover, Autosuggest
            ├── routing/     # Route calculation
            ├── isoline/     # Reachability polygons
            ├── matching/    # GPS trace matching
            ├── tour/        # Tour optimization
            ├── traffic/     # Flow & incidents
            ├── tiling/      # Vector tiles
            ├── positioning/ # Cell/Wi-Fi positioning
            ├── attributes/  # Map attributes
            └── imaging/     # Static map images
```

## Supported HERE APIs

| Domain | API | Endpoint | Tests |
|--------|-----|----------|-------|
| Search | Geocoding & Search v7 | `geocode.search.hereapi.com/v1` | 5 |
| Routing | Routing v8 | `router.hereapi.com/v8` | 1 |
| Isoline | Isoline Routing v8 | `isoline.router.hereapi.com/v8` | 2 |
| Matching | Route Matching v8 | `routematching.hereapi.com/v8` | 2 |
| Tour | Tour Planning v3 | `tourplanning.hereapi.com/v3` | 4 |
| Traffic | Traffic v7 | `data.traffic.hereapi.com/v7` | 3 |
| Tiling | Vector Tile v2 | `vector.hereapi.com/v2` | 3 |
| Positioning | Positioning v2 | `positioning.hereapi.com/v2` | 3 |
| Attributes | Map Attributes v8 | `smap.hereapi.com/v8` | 2 |
| Imaging | Map Image v3 | `image.maps.hereapi.com/mia/v3` | 2 |

**31 total contract tests, all passing.**

## Quick Start

```rust
use everymap_core::domains::routing::{Router, RouteRequest};
use everymap_core::auth::ApiKeyProvider;
use everymap_providers_here::domain::routing::{HereRouter, HereRouteOptions, TransportMode};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));

    let router = HereRouter::new(client);

    let request = RouteRequest {
        start: Coordinate::new(52.52, 13.405).unwrap(),
        end: Coordinate::new(52.54, 13.42).unwrap(),
        options: HereRouteOptions {
            transport_mode: Some(TransportMode::Car),
            ..Default::default()
        },
    };

    let response = router.calculate_route(request).await.unwrap();
    println!("Distance: {}m, Duration: {}s", response.distance, response.duration);
}
```

## Rich Provider Types

Each domain exposes HERE-specific rich types alongside the core trait:

```rust
use everymap_providers_here::domain::tour::{
    TourProblem, Fleet, VehicleType, Plan, Job, Objective,
};

// Build a rich tour planning problem
let problem = TourProblem {
    fleet: Fleet { types: vec![...], profiles: vec![...] },
    plan: Plan { jobs: vec![...], ..Default::default() },
    objectives: Some(vec![Objective::MinimizeCost]),
    ..Default::default()
};

// Use the rich API directly
let solution = planner.solve(problem).await?;
println!("Tours: {}, Unassigned: {}", solution.tours.len(), solution.unassigned.len());
```

## Design Principles

- **SOLID**: Core traits have zero knowledge of provider implementations.
- **Type-safe**: All API parameters and responses are strongly typed with serde.
- **Zero-cost**: Traits use associated types, not `dyn` dispatch.
- **TDD**: Contract tests using `wiremock` for every domain.
- **Full coverage**: All OpenAPI parameters and response fields are modeled.

## Build & Test

```bash
# Build
cargo build

# Run all tests
cargo test

# Lint
cargo clippy -- -D warnings
```

## Future Work

- OAuth2 authentication provider
- Additional provider crates (MapBox, TomTom, Google Maps)
- Criterion benchmarks for large response deserialization
- Comprehensive rustdoc examples
- CI/CD pipeline with GitHub Actions

## License

MIT