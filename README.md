# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs with provider abstraction.

## Overview

EveryMap-RS provides a unified interface for geospatial services across multiple providers. Currently supports **HERE Technologies** and **Google Maps** APIs with plans for MapBox and TomTom.

The architecture uses **domain-driven design** — each geospatial capability (routing, search, traffic, etc.) is defined as a trait in `everymap-core`, with provider-specific implementations in separate crates. Switch providers by changing one line of code.

## Architecture

```
everymap-rs/
├── everymap-core/              # Core traits, types, auth, error
│   └── src/
│       ├── domains/            # 10 domain traits + concrete Options/Response types
│       ├── types/              # Coordinate, BoundingBox, Address, Polyline
│       ├── auth/               # AuthProvider, ApiKeyProvider
│       ├── client/             # HttpClient trait + DefaultHttpClient
│       └── error/              # EveryMapError (structured errors)
├── everymap-providers-here/    # HERE Technologies implementation
│   └── src/
│       ├── client/             # HereClient (HTTP + auth)
│       ├── ext.rs              # Extension traits (discover, autosuggest, etc.)
│       └── domain/             # 10 domain implementations + From conversions
├── everymap-providers-google/  # Google Maps implementation
│   └── src/
│       ├── client.rs           # GoogleClient (HTTP + auth)
│       └── domain/
│           ├── search/         # GoogleGeocoder (Geocoding API)
│           ├── routing/        # GoogleRouter (Directions API)
│           └── unsupported.rs  # Stubs for unsupported domains
└── everymap-cli/               # CLI for interacting with providers
    └── src/
        ├── main.rs             # clap-based CLI with --provider flag
        ├── config.rs           # ~/.everymap/config.toml support
        └── output.rs           # JSON/Pretty/Summary output formats
```

## Provider Support

| Domain | Core Trait | HERE | Google |
|--------|-----------|------|--------|
| Search | `Geocoder` | Geocoding API | Geocoding API |
| Routing | `Router` | Routing v8 | Directions API |
| Isoline | `IsolineProvider` | Isoline Routing v8 | *Unsupported* |
| Matching | `RouteMatcher` | Route Matching v8 | *Unsupported* |
| Tour | `TourPlanner` | Tour Planning v3 | *Unsupported* |
| Traffic | `TrafficProvider` | Traffic v7 | *Unsupported* |
| Tiling | `TileProvider` | Vector Tile v2 | *Unsupported* |
| Positioning | `NetworkPositioner` | Positioning v2 | *Unsupported* |
| Attributes | `AttributeProvider` | Map Attributes v8 | *Unsupported* |
| Imaging | `MapImageProvider` | Map Image v3 | *Unsupported* |

Unsupported domains return a clear `UnsupportedDomain` error: `"google does not support traffic"`.

## Quick Start

### Library Usage

```rust
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::domains::routing::{Router, RouteOptions, TransportMode};
use everymap_core::auth::ApiKeyProvider;
use everymap_core::types::Coordinate;
use everymap_providers_here::client::HereClient;
use everymap_providers_here::domain::search::HereGeocoder;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "apiKey".to_string()));
    let client = Arc::new(HereClient::new(auth));

    // Use the trait for provider-agnostic code
    let geocoder: Box<dyn Geocoder> = Box::new(HereGeocoder::new(client));
    let result = geocoder.geocode("Berlin", &GeocodeOptions::default()).await.unwrap();

    for item in &result.items {
        println!("{}: ({}, {})", item.title.as_deref().unwrap_or("?"),
            item.coordinate.lat, item.coordinate.lng);
    }
}
```

Switch to Google by changing the provider:

```rust
use everymap_providers_google::client::GoogleClient;
use everymap_providers_google::GoogleGeocoder;

let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "key".to_string()));
let client = Arc::new(GoogleClient::new(auth));
let geocoder: Box<dyn Geocoder> = Box::new(GoogleGeocoder::new(client));
// Same trait, same response types — drop-in replacement
```

### CLI Usage

```bash
# HERE provider (default)
everymap --api-key $HERE_KEY geocode "Brandenburg Gate, Berlin"
everymap --api-key $HERE_KEY route --origin "52.52,13.405" --destination "52.54,13.42"

# Google provider
everymap --provider google --api-key $GOOGLE_KEY geocode "Brandenburg Gate, Berlin"
everymap --provider google --api-key $GOOGLE_KEY route --origin "52.52,13.405" --destination "52.54,13.42"

# Config file (~/.everymap/config.toml)
# [providers.here]
# api_key = "your-here-key"
# [providers.google]
# api_key = "your-google-key"

# Output formats
everymap geocode "Paris" --output json      # compact JSON
everymap geocode "Paris" --output pretty     # formatted JSON
everymap geocode "Paris" --output summary    # condensed human-readable

# All 11 commands
everymap geocode "Berlin"
everymap reverse-geocode --lat 52.52 --lng 13.40
everymap route --origin "52.52,13.40" --destination "52.54,13.42" --transport car
everymap traffic --lat 52.52 --lng 13.40
everymap position
everymap isoline --lat 52.52 --lng 13.40 --range 1000
everymap match-route --trace "52.5,13.3;52.6,13.4"
everymap tour --stops "52.5,13.3" "52.6,13.4"
everymap tile --z 14 --x 8800 --y 5374
everymap attributes --bbox "52.0,13.0,52.5,13.5"
everymap map-image --lat 52.52 --lng 13.40 --zoom 14
```

## Core Response Types

Enriched types that work across all providers:

```rust
pub struct SearchResult {
    pub id: Option<String>,
    pub coordinate: Coordinate,
    pub address: Address,              // structured: street, city, country, etc.
    pub title: Option<String>,
    pub result_type: SearchResultType, // ExactMatch, Approximate, etc.
    pub distance: Option<f64>,
    pub confidence: Option<f64>,
    pub categories: Vec<String>,
    pub bounding_box: Option<BoundingBox>,
    pub raw: Option<serde_json::Value>, // provider-specific escape hatch
}

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

Provider-specific methods are available via extension traits (e.g., `HereGeocoderExt::discover()`) or inherent methods.

## Design Principles

- **SOLID**: Core traits have zero knowledge of provider implementations.
- **Type-safe**: All API parameters and responses are strongly typed with serde.
- **Dynamic dispatch ready**: Concrete option types enable `Box<dyn Trait>` for runtime provider selection.
- **TDD**: 88 tests (52 unit + 31 contract + 5 error cases), all passing.
- **Full coverage**: All OpenAPI parameters and response fields are modeled.
- **Portable**: Enriched core types with `raw` escape hatch for provider-specific data.
- **From conversions**: All providers implement `From<ProviderType> for CoreType`.

## Build & Test

```bash
cargo build                              # Build all crates
cargo test                               # Run 88 tests
cargo clippy -- -D warnings              # Lint (must pass clean)
cargo run -p everymap-cli -- --help      # Run CLI
```

## Adding a New Provider

1. Create `everymap-providers-{name}/` with `Cargo.toml` depending on `everymap-core`
2. Implement `Client` (thin HTTP + auth wrapper — copy `GoogleClient` as template)
3. Implement supported domain traits (start with `Geocoder` + `Router`)
4. Add `From<ProviderType> for CoreType` conversions
5. Add stubs for unsupported domains returning `UnsupportedDomain`
6. Add factory functions + dispatch in CLI `main.rs`
7. Add provider section in CLI `config.rs`

## Future Work

- OAuth2 authentication provider
- MapBox and TomTom provider crates
- Google Roads API (route matching), Static Maps API (imaging)
- Criterion benchmarks for large response deserialization
- CI/CD pipeline with GitHub Actions

## License

MIT