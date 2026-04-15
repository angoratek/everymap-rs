# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Implements HERE Technologies, Google Maps, TomTom, MapBox, and Radar APIs.

## Architecture
- **Workspace**: `everymap-core` (traits, types, auth, error) + `everymap-providers-here` (HERE) + `everymap-providers-google` (Google) + `everymap-providers-tomtom` (TomTom) + `everymap-providers-mapbox` (MapBox) + `everymap-providers-radar` (Radar) + `everymap-cli` (CLI tool) + `everymap-bench` (benchmark framework)
- **Pattern**: Each domain trait in `everymap-core` uses concrete `Options` and `Response` types (not associated types). Provider implementations define their own rich types and map to the core trait via `From` conversions.
- **Dynamic dispatch**: Traits use concrete types enabling `Box<dyn Geocoder>` for runtime provider selection.
- **Each domain module** has its own `types.rs` submodule with provider-specific request/response types.
- **Shared geo types**: `HereLatLng` in HERE's `domain/geo.rs`, `GoogleLatLng` in Google's `domain/geo.rs`.
- **Core response types** are rich enough for most use cases, with an optional `raw: Option<serde_json::Value>` escape hatch for provider-specific data.
- **`From` trait conversions** are implemented for all domains: `HereSearchItem→SearchResult`, `HereRoute→RouteResult`, `GoogleGeocodeResult→SearchResult`, `GoogleRoute→RouteResult`, etc.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts between modules.
- **Extension traits** for provider-specific methods: `HereGeocoderExt`, `HereTrafficExt`, `HerePositionerExt`, `HereTourPlannerExt`, `HereAttributeExt`, `GooglePositionerExt`, `GoogleAttributeExt`, `TomTomGeocoderExt`, `TomTomTrafficExt`, `MapBoxGeocoderExt`, `MapBoxRouterExt`, `RadarGeocoderExt`, `RadarRouterExt`, `RadarSearchExt`, `RadarMatchingExt`.
- **Unsupported domains** return `EveryMapError::UnsupportedDomain { provider, domain }`.

## Key Conventions
- Use `async-trait` for all domain traits.
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding.
- Use `wiremock` for contract tests — mock API responses and validate deserialization.
- Each domain has a `const BASE_URL` for its API endpoint.
- Client wrappers (`HereClient`, `GoogleClient`) are thin HTTP + auth wrappers; domains construct their own full URLs.
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`.
- All optional response fields use `#[serde(default)]`.
- Helper functions (`add_option`, `add_option_ref`) for building query params from optional fields.
- Binary responses (tiling, imaging) use `response.bytes().await?.to_vec()` for body and `response.headers()` for content-type.
- Provider-specific rich types live in `types.rs` submodules within each domain.
- Core traits return enriched types; providers expose rich provider-specific methods alongside the trait impl.
- Error type `EveryMapError` has structured variants: `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
- Core options types have `provider_extra: Option<serde_json::Value>` escape hatch for provider-specific params.

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs 345+ tests (unit + contract + CLI integration + error cases)
- `cargo build` — verify compilation
- `cargo run -p everymap-cli -- --help` — run CLI

## Provider Quick Reference

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
| Geofencing | `GeofenceProvider` | stub | stub | stub | stub | `RadarGeofenceProvider` |
| Tracking | `TripTracker` | stub | stub | stub | stub | `RadarTripTracker` |
| Fraud | `FraudDetector` | stub | stub | stub | stub | `RadarFraudDetector` |

## Adding a New Provider
1. Create `everymap-providers-{name}/` crate with `Cargo.toml` depending on `everymap-core`
2. Create `client.rs` — thin HTTP + auth wrapper (copy `GoogleClient` as template; for Radar, use `Authorization` header instead of query param)
3. Create `domain/geo.rs` — shared lat/lng type
4. Implement supported domain traits (start with `Geocoder` + `Router`)
5. Add `From<ProviderX> for CoreType` conversions for all response types
6. Add stub implementations for unsupported domains returning `UnsupportedDomain`
7. Add factory functions in CLI `main.rs` and dispatch in `run_{provider}_commands()`
8. Add provider section in `config.rs` `Providers` struct
9. Add workspace member in root `Cargo.toml`

## CLI Usage
- `--provider here` (default) or `--provider google` or `--provider tomtom` or `--provider mapbox` or `--provider radar`
- `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
- `--output json|pretty|summary`
- `--verbose` / `-v` — show request URL (redacted key), raw response body, timing on stderr
- API key param name defaults: `apiKey` for HERE, `key` for Google, `key` for TomTom, `access_token` for MapBox, `Authorization` header for Radar
- 11 base commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image
- 8 Radar-specific commands: geofence-search, geofence-create, geofence-get, geofence-delete, trip-create, trip-update, trip-get, fraud-check
- Radar extension trait methods (not exposed via CLI): ip_geocode, autocomplete, validate_address, distance, matrix, search_places, search_geofences, match_route_with_attributes

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