# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Implements HERE Technologies, Google Maps, TomTom, MapBox, and Radar APIs across 10 geospatial domains.

## Architecture
- **Workspace** (8 crates): `everymap-core` (traits, types, auth, error, client) + `everymap-providers-here` + `everymap-providers-google` + `everymap-providers-tomtom` + `everymap-providers-mapbox` + `everymap-providers-radar` + `everymap-cli` (CLI tool, 11 commands) + `everymap-bench` (benchmark framework, all 10 domains)
- **10 Domain Traits** in `everymap-core/src/domains/`: search, routing, traffic, positioning, isoline, matching, tour, tiling, attributes, imaging
- **Pattern**: Each domain trait uses concrete `Options` and `Response` types (not associated types). Provider implementations define their own rich types and map to core via `From` conversions.
- **Dynamic dispatch**: Traits use concrete types enabling `Box<dyn Geocoder>` for runtime provider selection.
- **ProviderClient** in `everymap-core/src/client/` consolidates all HTTP client logic (request, request_json, post_json, redact_api_key, truncate_str). Provider crates wrap it with thin structs.
- **Unsupported domain macros** in `everymap-core/src/unsupported.rs` (7 macros): `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_tile!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`
- **CLI unified dispatch**: `ProviderRegistry` in `everymap-cli/src/provider.rs` holds `Box<dyn Trait>` for each domain. Single `run_commands()` function dispatches all 11 commands.
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
- Provider crates define `const PROVIDER_NAME: &str` in their client module for consistent naming in errors and logs.
- Workspace-level lints configured in root `Cargo.toml` (`[workspace.lints]`): `unsafe_code = "deny"`, clippy `all = warn`.
- MSRV declared as 1.75 in `[workspace.package]`.

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs 575+ tests (unit + contract + CLI integration + error cases)
- `cargo build` — verify compilation
- `cargo run -p everymap-cli -- --help` — run CLI
- See [TESTING.md](TESTING.md) for comprehensive testing guide (live API smoke testing, contract test patterns, API compatibility notes)

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

**31 real implementations** across 5 providers.

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
- Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand
- `--provider here` (default) or `--provider google` or `--provider tomtom` or `--provider mapbox` or `--provider radar`
- `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
- `--output json|pretty|summary`
- `--verbose` / `-v` — show request URL (redacted key), raw response body, timing on stderr
- Binary commands (`tile`, `map-image`) save to a file. Use `--output-file` to set the path (defaults: `tile.omv`, `map.png`)
- `match-route` requires `--transport` (car, truck, pedestrian, bicycle; default: car)
- `tour` supports `--departure` for ISO 8601 departure time (default: now)
- `tile` supports `--layer` (optional; HERE: base/core/hybrid, TomTom: basic/hybrid/labels, MapBox: no layer param) — default varies by provider; HERE uses its own tiling scheme (Berlin z14: x=4494, y=2832)
- **TomTom coordinate order**: TomTokyo static image uses `lng,lat` for center parameter (not `lat,lng` like HERE)
- API key param name defaults: `apiKey` for HERE, `key` for Google/TomTom, `access_token` for MapBox, `Authorization` header for Radar
- Use `--lng=VALUE` (with `=`) for negative longitudes to avoid CLI arg parsing issues
- 11 commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image

## Git Rules
- **Never commit without explicit user approval.** Always ask before committing.
- Do not push to remote unless explicitly asked.

## API Compatibility Notes
- TomTom reverse geocode returns `addresses` (not `results`), position as `"lat,lon"` string
- TomTom isoline uses `distanceBudgetInMeters`/`timeBudgetInSec` (not `distance`/`time`)
- TomTom match-route uses `lon,lat;lon,lat` format (longitude first, semicolon-separated); endpoint is `/snapToRoads/1/snap` (not `/snapToRoads/1`); needs `fields` param for `projectedPoints`
- TomTokyo tour endpoint is `/routing/waypointoptimization/1`; request body uses `waypoints`; response returns `optimizedOrder`
- TomTokyo static image `center` parameter uses `lng,lat` order (longitude first); TomTom tile layer names are `basic`/`hybrid`/`labels` (not `base`)
- MapBox search v6 puts data in `properties` (`full_address`, `name`, `coordinates`, `bbox`, `context`), not top-level `place_name`/`text`/`center`
- MapBox static image URL has no `.png` extension; default style is `streets-v12`
- Radar routing step fields are `snake_case` (`start_location`) while leg fields are `camelCase` (`startLocation`)
- HERE Route Matching API v8 `mode` parameter uses compound format: `fastest;car;traffic:disabled` (not just transport mode)
- HERE Route Matching API v8 transport modes: `car`, `carHov`, `truck`, `pedestrian`, `bicycle`, `bus`, `emergency`, `motorcycle`, `roadTrain` (camelCase)
- `TourOptions` has `transport_mode: Option<TransportMode>` field for providers that support it (MapBox uses it for profile selection: driving/walking/cycling)

## Known Issues (from comprehensive provider review)
- **Google routing**: Uses legacy Directions API (not the recommended Routes API v2).
- **TomTom traffic severity**: "moderate" maps to `Minor` (core `IncidentSeverity` has no `Moderate` variant).

## Fixed Issues (previously known)
- **HERE enum serialization**: Fixed — all 21 enums now use `camelCase` (or `lowercase`) `rename_all` to match HERE API expectations.
- **HERE search `X-Request-ID`**: Fixed — now sent as HTTP header instead of query parameter.
- **HERE matching `mode` parameter**: Fixed — now uses compound format `fastest;car;traffic:disabled`.
- **HERE routing/isoline vehicle options**: Fixed — scooter, truck, ev, fuel, driver, taxi, tolls, max_speed_on_segment now serialized to query params.
- **MapBox tour**: Fixed — now uses `transport_mode` from `TourOptions` instead of hardcoded `driving`.
- **MapBox routing language**: Fixed — no longer sends unsupported `language` parameter to Directions API v5.
- **TomTom traffic bbox**: Fixed — longitude offset now uses `cos(lat)` correction for meridian convergence.

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
- Don't use abbreviations in variable or method names. Use full, descriptive names: `config` not `cfg`, `format` not `fmt`, `options` not `opts`, `result`/`response` not `res`, `message` not `msg`, `coordinate` not `coord`, `destination` not `dest`, `transport_mode` not `tm`, `index` not `idx`, `value` not `val`, `accuracy` not `acc`, `distance` not `dist`, `duration` not `dur`. Exception: standard Rust conventions like `lat`/`lng`/`lon` for geospatial coordinates.