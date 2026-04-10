# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Implements HERE Technologies and Google Maps APIs with plans for MapBox and TomTom.

## Architecture
- **Workspace**: `everymap-core` (traits, types, auth, error) + `everymap-providers-here` (HERE) + `everymap-providers-google` (Google) + `everymap-cli` (CLI tool)
- **Pattern**: Each domain trait in `everymap-core` uses concrete `Options` and `Response` types (not associated types). Provider implementations define their own rich types and map to the core trait via `From` conversions.
- **Dynamic dispatch**: Traits use concrete types enabling `Box<dyn Geocoder>` for runtime provider selection.
- **Each domain module** has its own `types.rs` submodule with provider-specific request/response types.
- **Shared geo types**: `HereLatLng` in HERE's `domain/geo.rs`, `GoogleLatLng` in Google's `domain/geo.rs`.
- **Core response types** are rich enough for most use cases, with an optional `raw: Option<serde_json::Value>` escape hatch for provider-specific data.
- **`From` trait conversions** are implemented for all domains: `HereSearchItem→SearchResult`, `HereRoute→RouteResult`, `GoogleGeocodeResult→SearchResult`, `GoogleRoute→RouteResult`, etc.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts between modules.
- **Extension traits** for provider-specific methods: `HereGeocoderExt`, `HereTrafficExt`, `HerePositionerExt`, `HereTourPlannerExt`.
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
- `cargo test` — runs 88 tests (52 unit + 31 contract + 5 error cases)
- `cargo build` — verify compilation
- `cargo run -p everymap-cli -- --help` — run CLI

## Provider Quick Reference

| Domain | Core Trait | HERE | Google |
|--------|-----------|------|--------|
| Search | `Geocoder` | `HereGeocoder` | `GoogleGeocoder` |
| Routing | `Router` | `HereRouter` | `GoogleRouter` |
| Isoline | `IsolineProvider` | `HereIsoline` | stub (UnsupportedDomain) |
| Matching | `RouteMatcher` | `HereRouteMatcher` | stub |
| Tour | `TourPlanner` | `HereTourPlanner` | stub |
| Traffic | `TrafficProvider` | `HereTraffic` | stub |
| Tiling | `TileProvider` | `HereTileProvider` | stub |
| Positioning | `NetworkPositioner` | `HerePositioner` | stub |
| Attributes | `AttributeProvider` | `HereAttributeProvider` | stub |
| Imaging | `MapImageProvider` | `HereMapImageProvider` | stub |

## Adding a New Provider
1. Create `everymap-providers-{name}/` crate with `Cargo.toml` depending on `everymap-core`
2. Create `client.rs` — thin HTTP + auth wrapper (copy `GoogleClient` as template)
3. Create `domain/geo.rs` — shared lat/lng type
4. Implement supported domain traits (start with `Geocoder` + `Router`)
5. Add `From<ProviderX> for CoreType` conversions for all response types
6. Add stub implementations for unsupported domains returning `UnsupportedDomain`
7. Add factory functions in CLI `main.rs` and dispatch in `run_{provider}_commands()`
8. Add provider section in `config.rs` `Providers` struct
9. Add workspace member in root `Cargo.toml`

## CLI Usage
- `--provider here` (default) or `--provider google`
- `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
- `--output json|pretty|summary`
- API key param name defaults: `apiKey` for HERE, `key` for Google
- 11 commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image

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