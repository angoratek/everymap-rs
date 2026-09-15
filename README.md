# EveryMap-RS

[![CI](https://github.com/angoratek/everymap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/angoratek/everymap-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/everymap-core.svg)](https://crates.io/crates/everymap-core)
[![docs.rs](https://docs.rs/everymap-core/badge.svg)](https://docs.rs/everymap-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modular, type-safe Rust wrapper for geospatial APIs with provider abstraction.

## Overview

EveryMap-RS provides a unified interface for geospatial services across multiple providers — **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. The architecture uses **domain-driven design** with 10 geospatial capabilities, each defined as a trait in `everymap-core`, with provider-specific implementations in separate crates. Switch providers by changing one line of code.

**Capabilities covered**: geocoding, reverse geocoding, routing / directions, isolines / isochrones, map matching (snap-to-road), tour planning / waypoint optimization (TSP), traffic flow & incidents, map tiles, geodata attributes, and static map images.

## Architecture

```
everymap-rs/
├── everymap-core/              # Core traits, types, auth, error, client
│   └── src/
│       ├── domains/            # 10 domain traits + concrete Options/Response types
│       ├── types/              # Coordinate, BoundingBox, Address, Polyline
│       ├── auth/               # AuthProvider, ApiKeyProvider, HeaderAuthProvider
│       ├── client/             # ProviderClient (shared HTTP logic), HttpClient trait
│       ├── unsupported.rs      # 7 stub macros for unsupported domains
│       └── error/              # EveryMapError (structured errors)
├── everymap-providers-here/    # HERE Technologies (10 domains implemented)
├── everymap-providers-google/  # Google Maps (6 domains implemented)
├── everymap-providers-tomtom/  # TomTom (8 domains implemented)
├── everymap-providers-mapbox/  # MapBox (7 domains implemented)
├── everymap-providers-radar/   # Radar (4 domains: search, routing, matching, tour)
├── everymap-cli/               # CLI with 11 commands + unified ProviderRegistry
└── everymap-bench/             # Cross-provider benchmark framework (all 10 domains)
```

## Provider Support

| Domain | Core Trait | HERE | Google | TomTom | MapBox | Radar |
|--------|-----------|------|--------|--------|--------|-------|
| Search | `Geocoder` | ✅ | ✅ | ✅ | ✅ | ✅ |
| Routing | `Router` | ✅ | ✅ | ✅ | ✅ | ✅ |
| Isoline | `IsolineProvider` | ✅ | — | ✅ | ✅ | — |
| Matching | `RouteMatcher` | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tour | `TourPlanner` | ✅ | — | ✅ | ✅ | ✅ |
| Traffic | `TrafficProvider` | ✅ | — | ✅ | — | — |
| Tiling | `TileProvider` | ✅ | — | ✅ | ✅ | — |
| Positioning | `NetworkPositioner` | ✅ | ✅ | — | — | — |
| Attributes | `AttributeProvider` | ✅ | ✅ | — | — | — |
| Imaging | `MapImageProvider` | ✅ | ✅ | ✅ | ✅ | — |

✅ = real implementation, — = unsupported (stub or N/A). **35 real implementations** across 5 providers.

Unsupported domains return a clear `UnsupportedDomain` error: `"google does not support traffic"`.

## Quick Start

### Installation

EveryMap-RS is published on crates.io — all crates share one version cadence (currently **0.2.1**):

```bash
# Add the core + provider crates to your project
cargo add everymap-core everymap-providers-here

# Or install the CLI binary
cargo install everymap-cli
```

| Crate | Description |
|---|---|
| [`everymap-core`](https://crates.io/crates/everymap-core) ([docs](https://docs.rs/everymap-core)) | Traits, types, auth, errors, shared HTTP client |
| [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) ([docs](https://docs.rs/everymap-providers-here)) | HERE Technologies — 10 domains |
| [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) ([docs](https://docs.rs/everymap-providers-google)) | Google Maps — 6 domains |
| [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) ([docs](https://docs.rs/everymap-providers-tomtom)) | TomTom — 8 domains |
| [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) ([docs](https://docs.rs/everymap-providers-mapbox)) | MapBox — 7 domains |
| [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) ([docs](https://docs.rs/everymap-providers-radar)) | Radar — 4 domains |
| [`everymap-cli`](https://crates.io/crates/everymap-cli) ([docs](https://docs.rs/everymap-cli)) | CLI with 11 commands |

Prebuilt CLI binaries for 5 targets (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows) are attached to each [GitHub Release](https://github.com/angoratek/everymap-rs/releases).

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
use everymap_providers_google::domain::search::GoogleGeocoder;

let auth = Arc::new(ApiKeyProvider::new("YOUR_API_KEY".to_string(), "key".to_string()));
let client = Arc::new(GoogleClient::new(auth));
let geocoder: Box<dyn Geocoder> = Box::new(GoogleGeocoder::new(client));
// Same trait, same response types — drop-in replacement
```

### CLI Usage

> **Important:** Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand.
> Binary commands (`tile`, `map-image`) save to a file instead of printing to stdout. Use `--output-file` to set the path.

```bash
# HERE provider (default)
everymap --api-key $HERE_KEY geocode "Brandenburg Gate, Berlin"
everymap --api-key $HERE_KEY route --origin "52.52,13.405" --destination "52.54,13.42"

# Google provider
everymap --provider google --api-key $GOOGLE_KEY geocode "Brandenburg Gate, Berlin"

# TomTom, MapBox, Radar providers
everymap --provider tomtom --api-key $TOMTOM_KEY geocode "Berlin"
everymap --provider mapbox --api-key $MAPBOX_KEY route --origin "52.52,13.405" --destination "52.54,13.42"
everymap --provider radar --api-key $RADAR_KEY geocode "Berlin"

# Config file (~/.everymap/config.toml)
# [providers.here]
# api_key = "your-here-key"
# [providers.google]
# api_key = "your-google-key"

# Output formats — flag goes before the subcommand
everymap --output json --api-key $KEY geocode "Paris"       # compact JSON
everymap --output pretty --api-key $KEY geocode "Paris"     # formatted JSON
everymap --output summary --api-key $KEY geocode "Paris"    # condensed human-readable

# All 11 commands (examples use HERE provider)
everymap --api-key $KEY geocode "Berlin"
everymap --api-key $KEY reverse-geocode --lat 52.52 --lng 13.40
everymap --api-key $KEY route --origin "52.52,13.40" --destination "52.54,13.42" --transport car
everymap --api-key $KEY traffic --lat 52.52 --lng 13.40
everymap --api-key $KEY position
everymap --api-key $KEY isoline --lat 52.52 --lng 13.40 --range 1000
everymap --api-key $KEY match-route --trace "52.5164,13.3777;52.5170,13.3900;52.5175,13.3950" --transport car
everymap --api-key $KEY tour --stops "52.5,13.3" "52.6,13.4"
everymap --api-key $KEY tile --z 14 --x 4494 --y 2832
everymap --api-key $KEY attributes --bbox "52.4,13.2;52.6,13.5" --layer roads
everymap --api-key $KEY map-image --lat 52.52 --lng 13.40 --zoom 14
```

### Benchmarking

```bash
# Benchmark all domains against all configured providers
everymap-bench --here-key $HERE_KEY --google-key $GOOGLE_KEY

# List available domains
everymap-bench --list

# Benchmark a specific domain
everymap-bench --domain routing --here-key $HERE_KEY --tomtom-key $TOMTOM_KEY

# Output formats: table (default), json, markdown
everymap-bench --output json --api-key $KEY
```

## Core Response Types

Enriched types that work across all providers:

```rust
pub struct SearchResult {
    pub id: Option<String>,
    pub coordinate: Coordinate,
    pub address: Address,
    pub title: Option<String>,
    pub result_type: SearchResultType,
    pub distance: Option<f64>,
    pub confidence: Option<f64>,
    pub categories: Vec<String>,
    pub bounding_box: Option<BoundingBox>,
    pub raw: Option<serde_json::Value>,
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
- **TDD**: 623 tests (unit + contract + CLI integration + error cases + bench), all passing with nextest.
- **Full coverage**: All OpenAPI parameters and response fields are modeled.
- **Portable**: Enriched core types with `raw` escape hatch for provider-specific data.
- **From conversions**: All providers implement `From<ProviderType> for CoreType`.
- **Zero duplication**: Shared `ProviderClient`, `unsupported_*!` macros, unified `ProviderRegistry` dispatch.

## Build & Test

```bash
cargo build                              # Build all 8 workspace crates
cargo nextest run --all-features         # Run 623 tests (install: cargo install cargo-nextest)
cargo test                               # Alternative: standard test runner
cargo clippy -- -D warnings              # Lint (must pass clean)
cargo run -p everymap-cli -- --help      # Run CLI
```

See [TESTING.md](TESTING.md) for comprehensive testing guide including live API smoke tests, contract test patterns, and provider-specific API compatibility notes.

## Adding a New Provider

1. Create `everymap-providers-{name}/` with `Cargo.toml` depending on `everymap-core`
2. Create `client.rs` — thin wrapper around `ProviderClient` from core (copy `GoogleClient` as template)
3. Create `domain/geo.rs` — shared lat/lng type
4. Implement supported domain traits (start with `Geocoder` + `Router`)
5. Add `From<ProviderType> for CoreType` conversions
6. Add stubs for unsupported domains using `everymap_core::unsupported_*!` macros
7. Add provider to `ProviderRegistry` in `everymap-cli/src/provider.rs`
8. Add provider section in `everymap-cli/src/config.rs`
9. Add workspace member in root `Cargo.toml`
10. Add provider to `everymap-bench/src/benchmark.rs` `BenchProviders::new()`

## Future Work

- OAuth2 authentication provider
- Upgrade Google routing from legacy Directions API to Routes API v2
- Add `Moderate` variant to core `IncidentSeverity` (TomTom traffic)
- Configurable image size for the `map-image` command
- Wire core `avoid`/`alternatives` fields to Radar routing API (currently only via `provider_extra`)
- Provider client macro to reduce boilerplate across crates

## License

MIT