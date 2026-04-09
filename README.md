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
│       ├── domains/         # 10 domain traits + enriched response types
│       ├── types/           # Coordinate, BoundingBox, Address, Polyline, FlexiblePolyline
│       ├── auth/            # AuthProvider, ApiKeyProvider
│       └── error/           # EveryMapError (structured HTTP/auth/provider/rate-limit errors)
├── everymap-providers-here/ # HERE Technologies implementation
│   └── src/
│       ├── client/          # HereClient (HTTP + auth)
│       └── domain/          # 10 domain implementations
│           ├── geo.rs       # Shared HereLatLng type
│           ├── search/      # Geocode, Discover, Autosuggest + From conversions
│           ├── routing/     # Route calculation
│           ├── isoline/     # Reachability polygons
│           ├── matching/    # GPS trace matching
│           ├── tour/        # Tour optimization
│           ├── traffic/     # Flow & incidents
│           ├── tiling/      # Vector tiles
│           ├── positioning/ # Cell/Wi-Fi positioning
│           ├── attributes/  # Map attributes
│           └── imaging/     # Static map images
└── everymap-cli/            # CLI for interacting with providers
    └── src/
        └── main.rs          # clap-based CLI (geocode, route, traffic, isoline, position)
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

### Library Usage

```rust
use everymap_core::domains::routing::{Router, RouteRequest, RouteResult};
use everymap_core::auth::ApiKeyProvider;
use everymap_providers_here::domain::routing::{HereRouter, HereRouteOptions, TransportMode};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));

    let router = HereRouter::new(client);

    let request = RouteRequest {
        start: everymap_core::types::Coordinate::new(52.52, 13.405).unwrap(),
        end: everymap_core::types::Coordinate::new(52.54, 13.42).unwrap(),
        options: HereRouteOptions {
            transport_mode: TransportMode::Car,
            ..Default::default()
        },
    };

    let response = router.calculate_route(request).await.unwrap();
    for route in &response.routes {
        println!("Distance: {}m, Duration: {}s", route.distance, route.duration);
    }
}
```

### CLI Usage

```bash
# Geocode an address
everymap-cli --api-key $HERE_KEY geocode "Brandenburg Gate, Berlin"

# Reverse geocode
everymap-cli --api-key $HERE_KEY reverse-geocode --lat 52.5163 --lng 13.3777

# Calculate a route
everymap-cli --api-key $HERE_KEY route --origin "52.52,13.405" --destination "52.54,13.42"

# Get traffic data
everymap-cli --api-key $HERE_KEY traffic --lat 52.52 --lng 13.405

# Set API key via environment variable
export EVERYMAP_API_KEY=your_key_here
everymap-cli geocode "Paris, France"
```

## Core Response Types

EveryMap-RS provides enriched core types that work across all providers:

```rust
// SearchResult — rich enough for most use cases
pub struct SearchResult {
    pub id: Option<String>,
    pub coordinate: Coordinate,
    pub address: Address,           // structured: street, city, country, etc.
    pub title: Option<String>,
    pub result_type: SearchResultType, // ExactMatch, Approximate, etc.
    pub distance: Option<f64>,
    pub confidence: Option<f64>,
    pub categories: Vec<String>,
    pub bounding_box: Option<BoundingBox>,
    pub raw: Option<serde_json::Value>, // provider-specific escape hatch
}

// RouteResult — with steps, transport mode, bounding box
pub struct RouteResult {
    pub distance: f64,
    pub duration: f64,
    pub geometry: Polyline,
    pub transport_mode: Option<TransportMode>,
    pub steps: Vec<RouteStep>,
    pub bounding_box: Option<BoundingBox>,
    pub raw: Option<serde_json::Value>,
}
```

Provider-specific rich types are still accessible via inherent methods (e.g., `HereGeocoder::discover()`).

## Design Principles

- **SOLID**: Core traits have zero knowledge of provider implementations.
- **Type-safe**: All API parameters and responses are strongly typed with serde.
- **Zero-cost**: Traits use associated types, not `dyn` dispatch.
- **TDD**: Contract tests using `wiremock` for every domain.
- **Full coverage**: All OpenAPI parameters and response fields are modeled.
- **Portable**: Enriched core types with `raw` escape hatch for provider-specific data.
- **From conversions**: `From<HereAddress> for Address`, `From<HereSearchItem> for SearchResult`.

## Build & Test

```bash
# Build
cargo build

# Run all tests
cargo test

# Lint
cargo clippy -- -D warnings

# Run CLI
cargo run -p everymap-cli -- --help
```

## Future Work

- Injectable HTTP client trait (custom timeout/pool/proxy)
- everymap-cli Phase 2 (config file, table output, all domains)
- Extension traits for provider-specific methods
- everymap-providers-google (search + routing)
- OAuth2 authentication provider
- Dynamic provider registry for runtime dispatch
- Additional provider crates (MapBox, TomTom, Google Maps)
- Criterion benchmarks for large response deserialization
- Comprehensive rustdoc examples
- CI/CD pipeline with GitHub Actions

## License

MIT