# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Implements HERE Technologies, Google Maps, TomTom, MapBox, and Radar APIs across 13 geospatial domains.

## Architecture
- **Workspace** (8 crates): `everymap-core` (traits, types, auth, error, client) + `everymap-providers-here` + `everymap-providers-google` + `everymap-providers-tomtom` + `everymap-providers-mapbox` + `everymap-providers-radar` + `everymap-cli` (CLI tool, 19 commands) + `everymap-bench` (benchmark framework, all 13 domains)
- **13 Domain Traits** in `everymap-core/src/domains/`: search, routing, traffic, positioning, isoline, matching, tour, tiling, attributes, imaging, geofencing, tracking, fraud
- **Pattern**: Each domain trait uses concrete `Options` and `Response` types (not associated types). Provider implementations define their own rich types and map to core via `From` conversions.
- **Dynamic dispatch**: Traits use concrete types enabling `Box<dyn Geocoder>` for runtime provider selection.
- **ProviderClient** in `everymap-core/src/client/` consolidates all HTTP client logic (request, request_json, post_json, redact_api_key, truncate_str). Provider crates wrap it with thin structs.
- **Unsupported domain macros** in `everymap-core/src/unsupported.rs` (10 macros): `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_tile!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`, `unsupported_geofence!`, `unsupported_trip_tracker!`, `unsupported_fraud_detector!`
- **CLI unified dispatch**: `ProviderRegistry` in `everymap-cli/src/provider.rs` holds `Box<dyn Trait>` for each domain. Single `run_commands()` function dispatches all 19 commands.
- **Each domain module** has its own `types.rs` submodule with provider-specific request/response types.
- **Shared geo types**: `HereLatLng` in HERE's `domain/geo.rs`, `GoogleLatLng` in Google's `domain/geo.rs`.
- **Core response types** are rich enough for most use cases, with an optional `raw: Option<serde_json::Value>` escape hatch for provider-specific data.
- **`From` trait conversions** implemented for all domains.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts.
- **Extension traits** for provider-specific methods.

## Key Conventions
- Use `async-trait` for all domain traits.
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding.
- Use `wiremock` for contract tests — mock API responses and validate deserialization.
- Each domain has a `const BASE_URL` for its API endpoint.
- Provider client wrappers delegate to `everymap_core::client::ProviderClient`.
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`.
- All optional response fields use `#[serde(default)]`.
- Helper functions (`add_option`, `add_option_ref`) for building query params from optional fields.
- Binary responses (tiling, imaging) use `response.bytes().await?.to_vec()` for body and `response.headers()` for content-type.
- Provider-specific rich types live in `types.rs` submodules within each domain.
- Core traits return enriched types; providers expose rich provider-specific methods alongside the trait impl.
- Error type `EveryMapError` has structured variants: `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
- Core options types have `provider_extra: Option<serde_json::Value>` escape hatch for provider-specific params.
- Error body truncation in deserialization errors: 256 bytes max.
- API keys are zeroized on drop via `zeroize` crate (`#[zeroize(drop)]` on `ApiKeyProvider` and `HeaderAuthProvider`).
- CLI output writes directly to stdout via `serde_json::to_writer` (no intermediate `String` allocation for JSON/Pretty formats).

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs 540+ tests (unit + contract + CLI integration + error cases)
- `cargo build` — verify compilation
- `cargo run -p everymap-cli -- --help` — run CLI

## Provider Domain Coverage

| Domain | Core Trait | HERE | Google | TomTom | MapBox | Radar |
|--------|-----------|------|--------|--------|--------|-------|
| Search | `Geocoder` | `HereGeocoder` | `GoogleGeocoder` | `TomTomGeocoder` | `MapBoxGeocoder` | `RadarGeocoder` |
| Routing | `Router` | `HereRouter` | `GoogleRouter` | `TomTomRouter` | `MapBoxRouter` | `RadarRouter` |
| Isoline | `IsolineProvider` | `HereIsoline` | stub | `TomTomIsoline` | `MapBoxIsoline` | stub |
| Matching | `RouteMatcher` | `HereRouteMatcher` | `GoogleRouteMatcher` | `TomTomRouteMatcher` | `MapBoxRouteMatcher` | `RadarRouteMatcher` |
| Tour | `TourPlanner` | `HereTourPlanner` | stub | `TomTomTourPlanner` | `MapBoxTourPlanner` | `RadarTourPlanner` |
| Traffic | `TrafficProvider` | `HereTraffic` | stub | `TomTomTraffic` | stub | stub |
| Tiling | `TileProvider` | `HereTileProvider` | stub | `TomTomTileProvider` | `MapBoxTileProvider` | stub |
| Positioning | `NetworkPositioner` | `HerePositioner` | `GooglePositioner` | stub | stub | stub |
| Attributes | `AttributeProvider` | `HereAttributeProvider` | `GoogleAttributeProvider` | stub | stub | stub |
| Imaging | `MapImageProvider` | `HereMapImageProvider` | `GoogleMapImageProvider` | `TomTomMapImageProvider` | `MapBoxMapImageProvider` | stub |
| Geofencing | `GeofenceProvider` | N/A | N/A | N/A | N/A | `RadarGeofenceProvider` |
| Tracking | `TripTracker` | N/A | N/A | N/A | N/A | `RadarTripTracker` |
| Fraud | `FraudDetector` | N/A | N/A | N/A | N/A | `RadarFraudDetector` |

Implementation counts: HERE 10, Google 6, TomTom 8, MapBox 7, Radar 7 = **38 real implementations** across 5 providers.

## Adding a New Provider
1. Create `everymap-providers-{name}/` crate with `Cargo.toml` depending on `everymap-core`
2. Create `client.rs` — thin wrapper around `ProviderClient` from core (see `GoogleClient` as template)
3. Create `domain/geo.rs` — shared lat/lng type
4. Implement supported domain traits (start with `Geocoder` + `Router`)
5. Add `From<ProviderX> for CoreType` conversions for all response types
6. Add stub implementations using `everymap_core::unsupported_*!` macros
7. Add provider to `ProviderRegistry` in `everymap-cli/src/provider.rs`
8. Add provider section in `config.rs` `Providers` struct
9. Add workspace member in root `Cargo.toml`
10. Add provider to `everymap-bench/src/benchmark.rs` `BenchProviders::new()`

## CLI Usage
- `--provider here` (default) or `--provider google` or `--provider tomtom` or `--provider mapbox` or `--provider radar`
- `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
- `--output json|pretty|summary`
- `--verbose` / `-v` — show request URL (redacted key), raw response body, timing on stderr
- API key param name defaults: `apiKey` for HERE, `key` for Google/TomTom, `access_token` for MapBox, `Authorization` header for Radar
- 11 base commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image
- 8 Radar-specific commands: geofence-search, geofence-create, geofence-get, geofence-delete, trip-create, trip-update, trip-get, fraud-check

## Git Rules
- **Never commit without explicit user approval.** Always ask before committing.
- Do not push to remote unless explicitly asked.

## What NOT to Do
- Don't leak provider-specific types into `everymap-core`.
- Don't use glob re-exports (`pub use module::*`) in `domain/mod.rs` — use explicit re-exports.
- Don't add dependencies without workspace-level coordination.
- Don't skip `#[serde(default)]` on optional response fields.
- Don't return provider-specific types as `Self::Response` in trait impls — always convert to core types.
- Don't add `dyn` dispatch where generics suffice (but `Box<dyn Trait>` is fine for CLI runtime dispatch).
- Don't duplicate client logic — use `ProviderClient` from core.
- Don't write manual unsupported domain stubs — use `everymap_core::unsupported_*!` macros.
- Don't duplicate CLI command handlers — use `ProviderRegistry` for unified dispatch.