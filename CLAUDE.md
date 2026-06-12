# CLAUDE.md — Project Guidance for Claude Code

## Git Rules

- **Never commit without explicit user approval.** Always ask before committing.
- Do not push to remote unless explicitly asked.
- When allowed, create 1-liner commit messages, precise yet concise, e.g. "feat: move validate-cli.sh to scripts/ (374 assertions, timing, CI job), fix clippy redundant_field_names, optimize api_key clone."
- Never use the Co-author tag in commit messages.

### Commit Style

- **1-liner only** — precise, concise, no body paragraphs
- **Never** add `Co-Authored-By`, `Signed-off-by`, or any trailer/footer
- Format: `<type>: <imperative description>`
- Types: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `ci:`, `chore:`
- Examples:
  - `feat: add token-bucket rate limiter to ig-ratelimit`
  - `fix: map Graph API error code 2534022 to WindowExpired`
  - `test: add wiremock tests for all API error code mappings`

## Behavioral Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

Tradeoff: These guidelines bias toward caution over speed. For trivial tasks, use judgment.
1. Think Before Coding

Don't assume. Don't hide confusion. Surface tradeoffs.

Before implementing:

    State your assumptions explicitly. If uncertain, ask.
    If multiple interpretations exist, present them - don't pick silently.
    If a simpler approach exists, say so. Push back when warranted.
    If something is unclear, stop. Name what's confusing. Ask.

2. Simplicity First

Minimum code that solves the problem. Nothing speculative.

    No features beyond what was asked.
    No abstractions for single-use code.
    No "flexibility" or "configurability" that wasn't requested.
    No error handling for impossible scenarios.
    If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.
3. Surgical Changes

Touch only what you must. Clean up only your own mess.

When editing existing code:

    Don't "improve" adjacent code, comments, or formatting.
    Don't refactor things that aren't broken.
    Match existing style, even if you'd do it differently.
    If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

    Remove imports/variables/functions that YOUR changes made unused.
    Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.
4. Goal-Driven Execution

Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

    "Add validation" → "Write tests for invalid inputs, then make them pass"
    "Fix the bug" → "Write a test that reproduces it, then make it pass"
    "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

These guidelines are working if: fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

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
- Use `DepartureTime` enum for departure/arrival time fields — never `Option<String>` guessing.
- All variable names must be full words (no abbreviations). See "What NOT to Do" for the full list.

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs 624 tests (unit + contract + CLI integration + error cases + bench)
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

**35 real implementations** across 5 providers.

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
- `geocode` supports `--limit`, `--language`, `--country` (repeatable), `--bbox`
- `reverse-geocode` supports `--limit`, `--language`, `--radius`
- `route` supports `--transport` (car, truck, pedestrian, bicycle, scooter, bus, taxi), `--alternatives`, `--avoid` (repeatable: tolls, ferries, tunnels, highways, dirt-roads), `--departure-time`, `--arrival-time`, `--language`
- `match-route` supports `--transport` (car, truck, pedestrian, bicycle), `--heading`, `--departure-time`, `--avoid` (repeatable)
- `tour` supports `--departure` for ISO 8601 departure time (default: now), `--transport`
- `isoline` supports `--range-type` (distance, time), `--departure-time`, `--avoid` (repeatable)
- `traffic` supports `--radius`, `--include-incidents`, `--language`
- `tile` supports `--layer` (optional; HERE: base/core/hybrid, TomTom: basic/hybrid/labels, MapBox: no layer param), `--format` — HERE uses its own tiling scheme (Berlin z14: x=4494, y=2832)
- `attributes` supports `--layer`, `--format`, `--ids`, `--include`, `--language`
- `map-image` supports `--format`, `--language`
- **TomTom coordinate order**: TomTokyo static image uses `lng,lat` for center parameter (not `lat,lng` like HERE)
- API key param name defaults: `apiKey` for HERE, `key` for Google/TomTom, `access_token` for MapBox, `Authorization` header for Radar
- Use `--lng=VALUE` (with `=`) for negative longitudes to avoid CLI arg parsing issues
- 11 commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image

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
- **Radar routing**: Core `avoid` and `alternatives` fields only usable via `provider_extra` (core fields emit `log::warn!`).
- **Google matching**: `transport_mode`, `heading`, `departure_time`, `avoid` are all ignored (logged as warnings).

## Fixed Issues (previously known)
- **Silent parameter drops** (Phase 1): 24 locations across all 5 providers now emit `log::warn!` when core option fields are unsupported (Google: limit/radius/arrival_time/avoid/bbox/language; TomTom: avoid/tour/matching/traffic/isoline; MapBox: departure_time/arrival_time/avoid/format/tour; Radar: bounding_box/alternatives/avoid/arrival_time/transport_mode/tour/matching; HERE: reverse radius/traffic language/imaging format).
- **TomTom routing/isoline avoid** (Phase 2): Core `AvoidType` field now mapped to TomTom API params (Tolls→avoidTollRoads, Ferries→avoidFerries, Tunnels→avoidTunnels, Highways→avoidMotorways, DirtRoads→avoidUnpavedRoads).
- **HERE reverse geocode radius** (Phase 2): Core `radius` field now passed to HERE reverse geocode API.
- **HERE imaging format** (Phase 2): Core `format` field now read alongside `provider_extra`.
- **HERE traffic language** (Phase 2): `lang` param added to flow query from `options.language`.
- **Radar search bounding_box** (Phase 2): Center point from bounding box now passed as `near` param.
- **Google geocode limit** (Phase 2): Post-response truncation when `options.limit` is set.
- **MapBox imaging format** (Phase 2): Read from `options.format` core field.
- **Radar tour transport_mode** (Phase 2): Now read from `options.transport_mode` core field.
- **20+ CLI flags** (Phase 3): `--limit`, `--language`, `--country`, `--bbox`, `--radius`, `--alternatives`, `--avoid`, `--departure-time`, `--arrival-time`, `--heading`, `--range-type`, `--format` added across all 11 commands.
- **Google routing avoid types**: Fixed — removed invalid `indoor` mapping for DirtRoads, removed unsupported Tunnels from avoid parameter list. Uses typed `DepartureTime` match instead of string guessing.
- **HERE traffic incidents**: Fixed — `include_incidents` now actually fetches incidents (was always returning empty).
- **CLI Attributes --layer**: Fixed — key changed from singular `"layer"` to plural `"layers"` matching HERE provider.
- **HERE matching transport modes**: Fixed — Bus→Bus, Scooter→Motorcycle, Taxi→Taxi (were all mapped to Car).
- **DepartureTime enum**: Fixed — replaced fragile `Option<String>` guessing with typed `DepartureTime` enum (Now, Timestamp, Iso8601) across RouteOptions, MatchingOptions, IsolineOptions.
- **MapBox reverse geocode radius**: Fixed — now emits warning instead of silently ignoring radius.
- **Code abbreviations**: Fixed — all 823+ occurrences eliminated (`opts`→`options`, `res`→`response`, `coord`→`coordinate`, etc.).
- **HERE enum serialization**: Fixed — all 21 enums now use `camelCase` (or `lowercase`) `rename_all` to match HERE API expectations.
- **HERE search `X-Request-ID`**: Fixed — now sent as HTTP header instead of query parameter.
- **HERE matching `mode` parameter**: Fixed — now uses compound format `fastest;car;traffic:disabled`.
- **HERE routing/isoline vehicle options**: Fixed — scooter, truck, ev, fuel, driver, taxi, tolls, max_speed_on_segment now serialized to query params.
- **MapBox tour**: Fixed — now uses `transport_mode` from `TourOptions` instead of hardcoded `driving`.
- **MapBox routing language**: Fixed — no longer sends unsupported `language` parameter to Directions API v5 (now emits warning).
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