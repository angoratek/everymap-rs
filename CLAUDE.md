# CLAUDE.md — Project Guidance for Claude Code

## Git and commit rules

- **Never commit without explicit user approval.** Always ask before committing.
- Do not push to remote unless explicitly asked.
- **1-liner only** — precise, concise, no body paragraphs. Format `<type>: <imperative description>`; types `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `ci:`, `chore:`.
- **A maintainer commit carries no trailer**: never `Co-Authored-By`, `Signed-off-by` or any other. An outside contribution submitted through a pull request carries `Signed-off-by:` for the Developer Certificate of Origin, checked by `scripts/check-dco-signoff.mjs`; commits made directly on `main` by the maintainer stay one-line with no trailers.

## Parallel workstreams

- **Prefer parallelism for independent slices.** Give each independent track its own git worktree and branch (`Agent` with `isolation: "worktree"` or `git worktree add`); name branches `feat/<slug>`, `docs/<slug>`, `ci/<slug>`.
- **One tree per worker.** Never run `git stash`, `git checkout` or history rewrites in the shared tree.
- **Agree the seam before fan-out.** State method names, event field names, record shapes and file ownership in every brief; two agents must not edit the same file.
- **Small, justified fans only.** Delegate work that spans many files; do the spine yourself when it is a handful of tool calls.
- **Integrate deliberately.** Review each diff and verify its gates, then merge or cherry-pick onto `main`; the main session owns the final full gate on the merged tree.
- **Clean up as you go.** Remove an integrated worktree (`git worktree remove --force <path>`, then `git worktree prune`) and delete its branch; keep `.claude/worktrees/` empty between slices.
- **Branches stay local by default.** Push a feature branch or open a draft PR only when the owner asks; never merge a PR without owner approval.

## Behavioral guidelines

1. **Think before coding.** State assumptions, surface tradeoffs, and ask when a requirement is unclear or has multiple interpretations.
2. **Simplicity first.** Minimum code that solves the problem; no speculative features, abstractions, configurability or impossible-case handling.
3. **Surgical changes.** Touch only what the request needs, match existing style, remove only imports or variables your change orphaned, and mention unrelated dead code rather than deleting it.
4. **Goal-driven execution.** Turn each task into a verifiable goal, state a brief plan, and loop until it is verified.

## Work tracking and CHANGELOG.md

- The GitHub issue tracker is the work tracker: every open work item is one issue with a milestone and a label.
- Each milestone description in GitHub carries its done-when line.
- `CHANGELOG.md` records meaningful changes; a slice ends with its entry. Keep dates ISO-8601 and released sections newest-first.

## Coding conventions

1. Use the best known and shared practices for each language.
2. Follow SOLID: single responsibility, open-closed, Liskov substitution, interface segregation and dependency inversion.
3. No abbreviations in code; choose concise yet precise names for variables, methods, classes and packages.
4. Generate meaningful code documentation that automated docs can consume.
5. **No magic numbers**: every static value used in logic lives in a named, documented constant. Literals are acceptable only for structural math (`0.5` halving, NDC `[0,1]` conversions) and inside tests when local and self-explanatory.

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Implements HERE Technologies, Google Maps, TomTom, MapBox, Radar, MapQuest, Photon, OSRM and Valhalla APIs, plus URL-template raster and vector tilesets, behind sixteen core contracts.

## Architecture
- **Workspace** (14 members, 13 published): `everymap-core` (traits, types, auth, error, client) + `everymap-providers-here` + `-google` + `-tomtom` + `-mapbox` + `-radar` + `-mapquest` + `-photon` + `-osrm` + `-valhalla` + `-url-template-raster` + `-url-template-vector` + `everymap-cli` (CLI tool, 11 commands) + `everymap-bench` (benchmark framework, not published)
- **Contracts** in `everymap-core/src/domains/`: `Geocoder`, `PlaceSearchProvider`, `AutocompleteProvider`, `Router`, `DistanceMatrixProvider`, `TrafficProvider`, `RasterTileProvider`, `VectorTileProvider`, `ElevationProvider`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`, `OfflineRoutingProvider`
- **Targets**: `everymap-core` and every provider crate build for `wasm32-unknown-unknown`; the CLI and the bench are native applications. A provider reaches the network only through `ProviderClient`'s transport seam and reads time only through `web-time`; `scripts/check-provider-transport-isolation.sh` fails a provider that names `reqwest` or `tokio`, and CI lints the whole provider closure for wasm.
- **Pattern**: Each domain trait uses concrete `Options` and `Response` types (not associated types). Provider implementations define their own rich types and map to core via `From` conversions.
- **Dynamic dispatch**: Traits use concrete types enabling `Box<dyn Geocoder>` for runtime provider selection.
- **ProviderClient** in `everymap-core/src/client/` consolidates all HTTP client logic (request, request_json, post_json, redact_api_key, truncate_str). Provider crates wrap it with thin structs.
- **Unsupported domain macros** in `everymap-core/src/unsupported.rs` (9 macros): `unsupported_geocoder!`, `unsupported_router!`, `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_matcher!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`
- **CLI unified dispatch**: `ProviderRegistry` in `everymap-cli/src/provider.rs` holds `Box<dyn Trait>` for each domain. Single `run_commands()` function dispatches all 11 commands.
- **Each domain module** has its own `types.rs` submodule with provider-specific request/response types.
- **Core response types** are rich enough for most use cases, with an optional `raw: Option<serde_json::Value>` escape hatch for provider-specific data.
- **`From` trait conversions** implemented for all domains.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts.
- **Extension traits** for provider-specific methods.

## Key Conventions
- Use `async-trait` for all domain traits.
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding.
- Use `wiremock` for contract tests — mock API responses and validate deserialization.
- Each domain has a `const BASE_URL` for its API endpoint.
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`.
- All optional response fields use `#[serde(default)]`.
- Helper functions (`add_option`, `add_option_ref`) for building query params from optional fields.
- Provider-specific rich types live in `types.rs` submodules within each domain.
- Error type `EveryMapError` has structured variants: `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
- Core options types have `provider_extra: Option<serde_json::Value>` escape hatch for provider-specific params.
- Error body truncation in deserialization errors: 256 bytes max.
- API keys are zeroized on drop via `zeroize` crate (`#[zeroize(drop)]` on `ApiKeyProvider` and `HeaderAuthProvider`).
- CLI output writes directly to stdout via `serde_json::to_writer` (no intermediate `String` allocation for JSON/Pretty formats).
- Provider crates define `const PROVIDER_NAME: &str` in their client module for consistent naming in errors and logs.
- Workspace-level lints configured in root `Cargo.toml` (`[workspace.lints]`): `unsafe_code = "deny"`, clippy `all = warn`. MSRV is 1.86.
- Use `DepartureTime` enum for departure/arrival time fields — never `Option<String>` guessing.
- All variable names must be full words (no abbreviations). See "What NOT to Do" for the full list.

## Build & Test Commands
The full local gate before a commit: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace`, the browser-closure clippy below, `scripts/generate-readmes.sh --check`, `scripts/check-provider-transport-isolation.sh`, `scripts/check-open-source-hygiene.sh` and `scripts/validate-cli.sh --quiet`.
- `cargo clippy -p everymap-core -p everymap-providers-here -p everymap-providers-google -p everymap-providers-tomtom -p everymap-providers-mapbox -p everymap-providers-radar -p everymap-providers-mapquest -p everymap-providers-photon -p everymap-providers-osrm -p everymap-providers-valhalla -p everymap-providers-url-template-raster -p everymap-providers-url-template-vector --target wasm32-unknown-unknown --lib -- -D warnings` — the browser closure; keep it in step with the crate list in `.github/workflows/ci.yml`
- `cargo test --workspace` prints the current test count; never copy that number into a document. Six live-service tests stay ignored without a key.
- `cargo run -p everymap-cli -- --help` — run CLI
- See [TESTING.md](TESTING.md) for live API smoke testing, contract-test patterns and API compatibility notes

## Provider Domain Coverage

The table covers the five original providers; the full contract × crate matrix is in `README.md`.

| Domain | Core Trait | HERE | Google | TomTom | MapBox | Radar |
|--------|-----------|------|--------|--------|--------|-------|
| Search | `Geocoder` | `HereGeocoder` | `GoogleGeocoder` | `TomTomGeocoder` | `MapBoxGeocoder` | `RadarGeocoder` |
| Routing | `Router` | `HereRouter` | `GoogleRouter` | `TomTomRouter` | `MapBoxRouter` | `RadarRouter` |
| Isoline | `IsolineProvider` | `HereIsoline` | stub | `TomTomIsoline` | `MapBoxIsoline` | stub |
| Elevation | `ElevationProvider` | — | `GoogleElevationProvider` | — | — | — |
| Matching | `RouteMatcher` | `HereRouteMatcher` | `GoogleRouteMatcher` | `TomTomRouteMatcher` | `MapBoxRouteMatcher` | `RadarRouteMatcher` |
| Tour | `TourPlanner` | `HereTourPlanner` | stub | `TomTomTourPlanner` | `MapBoxTourPlanner` | `RadarTourPlanner` |
| Traffic | `TrafficProvider` | `HereTraffic` | stub | `TomTomTraffic` | stub | stub |
| Raster tiles | `RasterTileProvider` | — | — | `TomTomTileProvider` | — | — |
| Vector tiles | `VectorTileProvider` | `HereTileProvider` | — | — | `MapBoxTileProvider` | — |
| Positioning | `NetworkPositioner` | `HerePositioner` | `GooglePositioner` | stub | stub | stub |
| Attributes | `AttributeProvider` | `HereAttributeProvider` | `GoogleAttributeProvider` | stub | stub | stub |
| Imaging | `MapImageProvider` | `HereMapImageProvider` | `GoogleMapImageProvider` | `TomTomMapImageProvider` | `MapBoxMapImageProvider` | stub |

MapQuest also serves imaging through `MapQuestMapImageProvider`.

**49 real implementations** across eleven provider crates. The five original providers cover the table above; MapQuest serves geocoding, place search, autocomplete, routing, distance matrix, traffic and raster tiles; Photon serves geocoding, place search and type-ahead; OSRM and Valhalla serve routing; the URL-template raster and vector crates serve their tile contract with the named OSM US and OpenFreeMap vector sources.

## Provider crates and the browser
Every provider crate builds for `wasm32-unknown-unknown`. A provider that grows a direct `reqwest`/`tokio` dependency or reads `std::time` breaks that silently at build time or at runtime in the browser: route the request through `ProviderClient` and read time through `web-time`, and keep the crate in the browser closure in `.github/workflows/ci.yml`.

## Adding a New Provider
1. Create `everymap-providers-{name}/` crate with `Cargo.toml` depending on `everymap-core`. The provider source must not name `reqwest` or `tokio`; a dependency the generated native constructor needs is scoped to `cfg(not(target_arch = "wasm32"))`, as the keyless crates do
2. Create `client.rs` — use the `everymap_core::provider_client!` macro (see `GoogleClient` as template; provider-specific methods like Radar's `post_json` stay as inherent impls)
3. Create `domain/geo.rs` — shared lat/lng type
4. Implement supported domain traits (start with `Geocoder` + `Router`)
5. Add `From<ProviderX> for CoreType` conversions for all response types
6. Add stub implementations using `everymap_core::unsupported_*!` macros
7. Add provider to `ProviderRegistry` in `everymap-cli/src/provider.rs`
8. Add provider section in `config.rs` `Providers` struct
9. Add workspace member in root `Cargo.toml` and the crate to the browser closure in `.github/workflows/ci.yml`
10. Add provider to `everymap-bench/src/benchmark.rs` `BenchProviders::new()`
11. Add `everymap-providers-{name}/README.md` from `docs/readme/` and run `scripts/generate-readmes.sh`

## CLI Usage
- Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand
- `--provider` is one of `here` (default), `google`, `tomtom`, `mapbox`, `radar`, `mapquest`, `photon`, `osrm`, `valhalla`, `url-template-raster` or `url-template-vector`
- Each provider is a cargo feature in `everymap-cli`; the default carries all of them, and `--no-default-features --features here` builds a HERE-only binary
- `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
- `--output json|pretty|summary`
- `--verbose` / `-v` — show request URL (redacted key), raw response body, timing on stderr
- Binary commands (`tile`, `map-image`) save to a file. Use `--output-file` to set the path (defaults: `tile.png` for raster, `tile.mvt` for vector, `map.png`)
- `tile` takes `--variant raster|vector`; with no variant it uses the first tile contract the provider serves. The URL-template providers take `--url-template` plus `--attribution`. HERE uses its own tiling scheme (Berlin z14: x=4494, y=2832)
- **TomTom coordinate order**: the static image uses `lng,lat` for `center` (not `lat,lng` like HERE)
- API key param name defaults: `apiKey` for HERE, `key` for Google/TomTom, `access_token` for MapBox, `Authorization` header for Radar
- Use `--lng=VALUE` (with `=`) for negative longitudes to avoid CLI arg parsing issues
- 11 commands: geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image

## API Compatibility Notes
- TomTom reverse geocode returns `addresses` (not `results`), position as `"lat,lon"` string
- TomTom isoline uses `distanceBudgetInMeters`/`timeBudgetInSec` (not `distance`/`time`)
- TomTom match-route uses `lon,lat;lon,lat` format (longitude first, semicolon-separated); endpoint is `/snapToRoads/1/snap` (not `/snapToRoads/1`); needs `fields` param for `projectedPoints`
- TomTom tour endpoint is `/routing/waypointoptimization/1`; request body uses `waypoints`; response returns `optimizedOrder`
- TomTom static image `center` parameter uses `lng,lat` order (longitude first); TomTom tile layer names are `basic`/`hybrid`/`labels` (not `base`)
- MapBox search v6 puts data in `properties` (`full_address`, `name`, `coordinates`, `bbox`, `context`), not top-level `place_name`/`text`/`center`
- MapBox static image URL has no `.png` extension; default style is `streets-v12`
- Radar routing step fields are `snake_case` (`start_location`) while leg fields are `camelCase` (`startLocation`)
- HERE Route Matching API v8 `mode` parameter uses compound format: `fastest;car;traffic:disabled` (not just transport mode)
- HERE Route Matching API v8 transport modes: `car`, `carHov`, `truck`, `pedestrian`, `bicycle`, `bus`, `emergency`, `motorcycle`, `roadTrain` (camelCase)
- `TourOptions` has `transport_mode: Option<TransportMode>` field for providers that support it (MapBox uses it for profile selection: driving/walking/cycling)
- Google routing uses Routes API v2: POST JSON body to `routes.googleapis.com/directions/v2:computeRoutes` with camelCase fields, `X-Goog-FieldMask` header required, API key as `key` query param (header alternative documented by Google), durations are `"1234s"` strings
- Radar Directions `avoid` accepts a comma-separated list of `tolls`, `highways`, `ferries`, `borderCrossings`; `alternatives` is boolean (two-location requests only)

## Known Issues (from comprehensive provider review)
- **Google matching**: `transport_mode`, `heading`, `departure_time`, `avoid` are all ignored (logged as warnings).
- **Radar routing avoid**: Tunnels and dirt-roads avoid types have no Radar API counterpart (Radar supports tolls, highways, ferries, borderCrossings) — they log a warning and are dropped.

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