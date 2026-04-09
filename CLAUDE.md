# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Currently implements HERE Technologies APIs with plans for MapBox, TomTom, and Google.

## Architecture
- **Workspace**: `everymap-core` (traits, types, auth, error) + `everymap-providers-here` (HERE implementations) + `everymap-cli` (CLI tool)
- **Pattern**: Each domain trait in `everymap-core` has associated `Options` and `Response` types. Provider implementations define their own rich types and map to the core trait via `From` conversions.
- **Each domain module** has its own `types.rs` submodule with full HERE-specific request/response types.
- **Shared geo types**: `HereLatLng` is in `domain/geo.rs` (shared across search, routing, etc.)
- **Core response types** are rich enough for most use cases, with an optional `raw: Option<serde_json::Value>` escape hatch for provider-specific data.
- **`From` trait conversions** are implemented for `HereAddress -> Address`, `HereSearchItem -> SearchResult`, `HereLatLng -> Coordinate`.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts between modules.

## Key Conventions
- Use `async-trait` for all domain traits.
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding.
- Use `wiremock` for contract tests — mock HERE API responses and validate deserialization.
- Each domain has a `const BASE_URL` for its API endpoint.
- `HereClient` is a thin HTTP + auth wrapper; domains construct their own full URLs.
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`.
- All optional response fields use `#[serde(default)]`.
- Helper functions (`add_option`, `add_option_ref`) for building query params from optional fields.
- Binary responses (tiling, imaging) use `response.bytes().await?.to_vec()` for body and `response.headers()` for content-type.
- Provider-specific rich types live in `types.rs` submodules within each domain.
- Core traits return enriched types (`SearchResult`, `RouteResult`, `TrafficFlow`, etc.); providers expose rich HERE-specific methods alongside the trait impl.
- Error type `EveryMapError` has structured variants: `HttpError { status, message, body }`, `AuthError { provider, message }`, `ProviderError { provider, code, message }`, `RateLimited { provider, retry_after_secs }`, etc.
- `Address` is a structured core type with fields like `label`, `street`, `city`, `country_code`, etc.

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs all 31 contract tests
- `cargo build` — verify compilation
- `cargo run -p everymap-cli -- --help` — run CLI

## Domain Module Layout
Each domain follows this pattern:
```
src/domain/
  ├── geo.rs            # Shared HereLatLng type
  ├── mod.rs            # Module declarations + explicit re-exports
  ├── search/           # HereGeocoder implementation
  │   ├── mod.rs        # Trait impl + helper functions
  │   └── types.rs      # HereSearchItem, HereAddress, etc.
  ├── routing/           # HereRouter implementation
  │   ├── mod.rs
  │   └── types.rs
  ... (one per domain)
```

## Core Response Types (enriched)
- `SearchResult`: id, coordinate, address (structured), title, result_type (enum), distance, confidence, categories, bounding_box, raw
- `RouteResponse`: routes (Vec<RouteResult>), each with distance, duration, geometry, transport_mode, steps, bounding_box, raw
- `TrafficResponse`: flows (Vec<TrafficFlow>), incidents (Vec<TrafficIncident>), raw
- `IsolineResponse`: isolines (Vec<IsolineResult>), raw
- `TraceResponse`: matched_points (Vec<MatchedPoint>), distance, duration, raw
- `TourResponse`: stops (Vec<TourStop>), total_distance, total_duration, unassigned_count, raw
- `PositioningResponse`: coordinate, accuracy, altitude, altitude_accuracy, raw

## Base URLs
- Search: `https://geocode.search.hereapi.com/v1`
- Routing: `https://router.hereapi.com/v8`
- Isoline: `https://isoline.router.hereapi.com/v8`
- Matching: `https://routematching.hereapi.com/v8`
- Tour: `https://tourplanning.hereapi.com/v3`
- Traffic: `https://data.traffic.hereapi.com/v7`
- Tiling: `https://vector.hereapi.com/v2`
- Positioning: `https://positioning.hereapi.com/v2`
- Attributes: `https://smap.hereapi.com/v8`
- Imaging: `https://image.maps.hereapi.com/mia/v3`

## Testing Pattern
- Test files: `everymap-providers-here/tests/{domain}_contract.rs`
- Each test creates a `MockServer`, defines expected JSON responses, verifies deserialization and trait behavior.
- Tests use `with_base_url()` to point at the mock server.
- Auth: `Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()))`

## CLI
- Binary: `everymap-cli`
- Commands: `geocode`, `reverse-geocode`, `route`, `traffic`, `position`, `isoline`
- Auth: `--api-key` or `EVERYMAP_API_KEY` env var
- Provider: `--provider here` (currently only HERE supported)

## Git Rules
- **Never commit without explicit user approval.** Always ask before committing. Do not assume the user wants a commit after making changes.
- Do not push to remote unless explicitly asked.

## What NOT to Do
- Don't add `dyn` dispatch where generics suffice (zero-cost abstractions).
- Don't leak HERE-specific types into `everymap-core`.
- Don't use glob re-exports (`pub use module::*`) in `domain/mod.rs` — use explicit re-exports to avoid conflicts.
- Don't add dependencies without workspace-level coordination.
- Don't skip the `#[serde(default)]` on optional response fields.
- Don't return HERE-specific types as `Self::Response` in trait impls — always convert to core types.