# EveryMap-RS

[![CI](https://github.com/angoratek/everymap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/angoratek/everymap-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/everymap-core.svg)](https://crates.io/crates/everymap-core)
[![docs.rs](https://docs.rs/everymap-core/badge.svg)](https://docs.rs/everymap-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modular, type-safe Rust wrapper for geospatial APIs with provider abstraction.

## Overview

EveryMap-RS provides one trait-based interface over eleven provider crates — **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset, including the OpenStreetMap US and OpenFreeMap tilesets. Business logic targets the trait, and a provider is a constructor change rather than a rewrite.

**Contracts**: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing (ordered waypoints, legs, maneuver types, instruction and speech text, lane guidance, geometry), distance matrices, isolines, map matching, tour planning, traffic flow and incidents, raster and vector tiles, elevation, geodata attributes, static map images, and offline routing.

**Targets**: `everymap-core` and every provider crate build for `wasm32-unknown-unknown` as well as native targets. Each provider reaches the network only through the shared client's transport seam, which is `reqwest` natively and `window.fetch` in a browser; no provider crate names `reqwest` or `tokio` itself. The CLI and the benchmark harness are native applications.

## Architecture

```
everymap-rs/
├── everymap-core/                          # Contracts, types, auth, error, shared client
│   └── src/
│       ├── domains/                        # Traits + concrete Options/Response types
│       ├── types/                          # Coordinate, BoundingBox, Address, Polyline
│       ├── auth/                           # AuthProvider, ApiKeyProvider, HeaderAuthProvider, OAuth2
│       ├── client/                         # ProviderClient (pacing, retries, timeouts, observers, web-time clock)
│       ├── http/                           # HttpRequest/HttpResponse/HttpTransport seam (reqwest, fetch)
│       ├── unsupported.rs                  # Stub macros for unsupported domains
│       └── error/                          # EveryMapError
├── everymap-providers-here/                # HERE Technologies
├── everymap-providers-google/              # Google Maps
├── everymap-providers-tomtom/              # TomTom
├── everymap-providers-mapbox/              # MapBox
├── everymap-providers-radar/               # Radar
├── everymap-providers-mapquest/            # MapQuest — geocoding, search, routing, matrix, traffic, tiles
├── everymap-providers-photon/              # Photon — keyless geocoding, place search, type-ahead
├── everymap-providers-osrm/                # OSRM routing
├── everymap-providers-valhalla/            # Valhalla routing
├── everymap-providers-url-template-raster/ # Any {z}/{x}/{y} raster source
├── everymap-providers-url-template-vector/ # Any {z}/{x}/{y} vector source, with named OSM US and OpenFreeMap sources
├── everymap-cli/                           # CLI with 11 commands + unified ProviderRegistry
└── everymap-bench/                         # Cross-provider benchmark framework
```

## Provider Support

The five full-service providers:

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

The crates built on the newest contracts:

| Contract | Trait | MapQuest | OSRM | Valhalla | URL-template raster | URL-template vector |
|----------|-------|----------|------|----------|---------------------|---------------------|
| Geocoding | `Geocoder` | ✅ | — | — | — | — |
| Place search | `PlaceSearchProvider` | ✅ | — | — | — | — |
| Type-ahead | `AutocompleteProvider` | ✅ | — | — | — | — |
| Routing | `Router` | ✅ | ✅ | ✅ | — | — |
| Distance matrix | `DistanceMatrixProvider` | ✅ | — | — | — | — |
| Traffic | `TrafficProvider` | ✅ | — | — | — | — |
| Raster tiles | `RasterTileProvider` | ✅ | — | — | ✅ | — |
| Vector tiles | `VectorTileProvider` | — | — | — | — | ✅ |

Photon serves the same search family without a credential:

| Contract | Trait | Photon |
|----------|-------|--------|
| Geocoding | `Geocoder` | ✅ |
| Place search | `PlaceSearchProvider` | ✅ |
| Type-ahead | `AutocompleteProvider` | ✅ |

✅ = real implementation, — = unsupported (stub or N/A).

An unsupported domain returns a clear `UnsupportedDomain` error: `"google does not support traffic"`.

## Quick Start

### Installation

EveryMap-RS is published on crates.io — all crates share one version cadence (currently **0.3.1**):

```bash
# Add the core + provider crates to your project
cargo add everymap-core everymap-providers-here

# Or install the CLI binary
cargo install everymap-cli
```

| Crate | Description |
|---|---|
| [`everymap-core`](https://crates.io/crates/everymap-core) ([docs](https://docs.rs/everymap-core)) | Contracts, types, auth, errors, shared client and transport seam |
| [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) ([docs](https://docs.rs/everymap-providers-here)) | HERE Technologies — 10 domains |
| [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) ([docs](https://docs.rs/everymap-providers-google)) | Google Maps — 6 domains |
| [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) ([docs](https://docs.rs/everymap-providers-tomtom)) | TomTom — 8 domains |
| [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) ([docs](https://docs.rs/everymap-providers-mapbox)) | MapBox — 7 domains |
| [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) ([docs](https://docs.rs/everymap-providers-radar)) | Radar — 4 domains |
| [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) ([docs](https://docs.rs/everymap-providers-mapquest)) | MapQuest — geocoding, place search, type-ahead, routing, matrix, traffic, raster tiles |
| [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) ([docs](https://docs.rs/everymap-providers-photon)) | Photon — keyless geocoding, place search and type-ahead |
| [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) ([docs](https://docs.rs/everymap-providers-osrm)) | OSRM — navigation-grade routing |
| [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) ([docs](https://docs.rs/everymap-providers-valhalla)) | Valhalla — navigation-grade routing |
| [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) ([docs](https://docs.rs/everymap-providers-url-template-raster)) | Any `{z}/{x}/{y}` raster source |
| [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) ([docs](https://docs.rs/everymap-providers-url-template-vector)) | Any `{z}/{x}/{y}` vector source, plus named OSM US and OpenFreeMap sources |
| [`everymap-cli`](https://crates.io/crates/everymap-cli) ([docs](https://docs.rs/everymap-cli)) | CLI with 11 commands |

Prebuilt CLI binaries for 5 targets (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows) are attached to each [GitHub Release](https://github.com/angoratek/everymap-rs/releases).

### Library Usage

```rust
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_core::auth::ApiKeyProvider;
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

A keyless provider such as OSRM takes a deployment address instead of a key, and a URL-template source takes its template and descriptor:

```rust
use everymap_core::domains::routing::{Router, RouteOptions};
use everymap_core::types::Coordinate;
use everymap_providers_osrm::{OsrmProviderConfiguration, OsrmRouter};

let router = OsrmRouter::new(OsrmProviderConfiguration {
    base_url: "https://router.project-osrm.org".to_string(),
    ..Default::default()
});
let route = router
    .calculate_route(
        &Coordinate::new(52.52, 13.405).unwrap(),
        &Coordinate::new(52.54, 13.42).unwrap(),
        &RouteOptions::default(),
    )
    .await?;
```

### CLI Usage

> **Important:** Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand.
> Binary commands (`tile`, `map-image`) save to a file instead of printing to stdout. Use `--output-file` to set the path.

The CLI selects every provider the workspace ships except the URL-template crates, which are library sources: a `--provider` value the binary does not carry is refused, and a command a provider cannot serve answers with its unsupported-domain error.

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

# MapQuest
everymap --provider mapquest --api-key $MAPQUEST_KEY geocode "Denver, CO"

# OSRM and Valhalla are keyless: the credential slot carries the deployment
# base URL, and an empty one reaches the documented local default.
everymap --provider osrm --api-key https://router.project-osrm.org route --origin "52.52,13.405" --destination "52.54,13.42"
everymap --provider valhalla route --origin "52.52,13.405" --destination "52.54,13.42"

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
    pub quality: Option<SearchQuality>,
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
    pub legs: Vec<RouteLeg>,
    pub bounding_box: Option<BoundingBox>,
    pub raw: Option<serde_json::Value>,
}
```

Provider-specific methods are available via extension traits (e.g., `HereGeocoderExt::discover()`) or inherent methods.

## Design Principles

- **SOLID**: Core traits have zero knowledge of provider implementations.
- **Type-safe**: All API parameters and responses are strongly typed with serde.
- **Dynamic dispatch ready**: Concrete option types enable `Box<dyn Trait>` for runtime provider selection.
- **One transport seam**: Every provider sends requests through `ProviderClient`, so one crate serves native and browser targets; a provider never names `reqwest` or `tokio`, and CI enforces it.
- **One clock**: Pacing, retries and token lifetimes read `web-time`, which is the browser's clock on `wasm32` and the system clock elsewhere.
- **TDD**: 939 tests (unit, contract, CLI integration, error cases, bench) across 97 suites.
- **Portable**: Enriched core types with a `raw` escape hatch for provider-specific data.
- **From conversions**: All providers implement `From<ProviderType> for CoreType`.
- **Zero duplication**: Shared `ProviderClient`, `unsupported_*!` macros, unified `ProviderRegistry` dispatch.

## Build & Test

```bash
cargo build                              # Build all 13 workspace crates
cargo nextest run --all-features         # Run the full suite (install: cargo install cargo-nextest)
cargo test                               # Alternative: standard test runner
cargo clippy -- -D warnings              # Lint (must pass clean)
cargo run -p everymap-cli -- --help      # Run CLI
```

See [TESTING.md](TESTING.md) for comprehensive testing guide including live API smoke tests, contract test patterns, and provider-specific API compatibility notes.

## Adding a New Provider

1. Create `everymap-providers-{name}/` with a `Cargo.toml` depending on `everymap-core` only.
2. Create `client.rs` — a `provider_client!` declaration over the shared `ProviderClient` (copy `GoogleClient` as the template). Do not add `reqwest` or `tokio`: the shared client already picks the target's transport.
3. Implement the contracts the service serves, mapping its vocabulary onto the core types (`SearchQuality`, `CongestionLevel`, `ManeuverType`, typed tiles) and its failures onto `EveryMapError`.
4. Add recorded fixtures and contract tests; no test may reach the network.
5. Add the crate to the root `Cargo.toml` members and to the browser closure in `.github/workflows/ci.yml`.
6. Add `everymap-providers-{name}/README.md` from `docs/readme/` and regenerate with `scripts/generate-readmes.sh`.
7. To make it selectable from the tools, add a `ProviderRegistry` arm in `everymap-cli/src/provider.rs`, a section in `everymap-cli/src/config.rs`, and a `BenchProviders` arm in `everymap-bench`.

## License

MIT
