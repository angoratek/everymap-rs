# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, Google, TomTom, MapBox, Radar) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace (8 crates)

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Address`, `Polyline`, `FlexiblePolyline`.
    - 10 Domain Traits: `Geocoder`, `Router`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `TrafficProvider`, `TileProvider`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`.
    - Core Options Types: `GeocodeOptions`, `RouteOptions`, etc. with `provider_extra: Option<serde_json::Value>` escape hatch.
    - Enriched Response Types: `SearchResult`, `RouteResult`, `TrafficFlow`, `TrafficIncident`, `IsolineResult`, `MatchedPoint`, `TourResponse`, etc.
    - Auth Traits: `AuthProvider` (`ApiKeyProvider`, `HeaderAuthProvider`, future `OAuth2Provider`).
    - Error System: Structured `EveryMapError` with `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
    - `ProviderClient`: Consolidated HTTP client logic (request, request_json, post_json, redact_api_key, truncate_str).
    - 7 Unsupported Domain Macros: `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_tile!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`.
- **`everymap-providers-here`**: HERE Technologies — 10 domains implemented (all common domains).
- **`everymap-providers-google`**: Google Maps — 6 domains implemented (search, routing, matching, positioning, attributes, imaging).
- **`everymap-providers-tomtom`**: TomTom — 8 domains implemented (search, routing, traffic, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-mapbox`**: MapBox — 7 domains implemented (search, routing, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-radar`**: Radar — 4 domains (search, routing, matching, tour).
- **`everymap-cli`**: CLI tool with 11 commands, unified `ProviderRegistry` dispatch.
- **`everymap-bench`**: Cross-provider benchmark framework covering all 10 domains with typed `ScenarioParams` dispatch.

### 2. Domain Abstraction

| Domain | Core Trait | HERE | Google | TomTom | MapBox | Radar |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Search** | `Geocoder` | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Routing** | `Router` | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Isolines** | `IsolineProvider` | ✅ | stub | ✅ | ✅ | stub |
| **Matching** | `RouteMatcher` | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Tour** | `TourPlanner` | ✅ | stub | ✅ | ✅ | ✅ |
| **Traffic** | `TrafficProvider` | ✅ | stub | ✅ | stub | stub |
| **Tiling** | `TileProvider` | ✅ | stub | ✅ | ✅ | stub |
| **Positioning** | `NetworkPositioner` | ✅ | ✅ | stub | stub | stub |
| **Attributes** | `AttributeProvider` | ✅ | ✅ | stub | stub | stub |
| **Imaging** | `MapImageProvider` | ✅ | ✅ | ✅ | ✅ | stub |

**35 real implementations** across 5 providers.

### 3. Key Design Decisions

- **Concrete options types** (not associated types): Enables `Box<dyn Geocoder>` dynamic dispatch.
- **Core response types are rich enough for 80% of use cases**: All fields are `Option<T>`. `raw: Option<serde_json::Value>` escape hatch for the remaining 20%.
- **Extension traits** for provider-specific methods: `HereGeocoderExt::discover()`, etc.
- **Feature flags** (planned): Gate providers behind compile-time features in CLI to reduce binary size.
- **Version independently**: Each crate follows its own semver pace.
- **ProviderClient consolidation**: All 5 provider clients delegate to shared `ProviderClient` from core.
- **Unsupported domain macros**: 7 macros eliminate boilerplate stubs.
- **Unified CLI dispatch**: `ProviderRegistry` replaces 5 duplicated command handler functions.

### 4. Authentication Layer
- **`AuthProvider` Trait**: `async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder>`
- **`ApiKeyProvider`**: Injects API key as query param (HERE, Google, TomTom, MapBox).
- **`HeaderAuthProvider`**: Injects API key as Authorization header (Radar).
- **`OAuth2Auth`** (planned): Token caching/refresh logic.

---

## Implementation History

### Phase 0: Architecture Foundations ✅
- Per-domain base URLs, FlexiblePolyline decoder, Core trait redesign, Domain mod.rs reorganized

### Phase 1–10: All 10 HERE Domains ✅
- All domains implemented with full parameter coverage, 31 contract tests

### Phase 11: Multi-Provider Architecture ✅
- Enriched core response types, `From` conversions, shared geo types, enriched error types, CLI Phase 1

### Phase 12: Concrete Options Refactor ✅
- All traits migrated to concrete Options types, `HttpClient` trait, extension traits, all 11 CLI commands, config file support, 28 unit tests in core

### Phase 13: From Conversions & Testing ✅
- `From` trait conversions for all domain types, 52 unit tests, 31 contract tests, CLI config file, HTTP error handling

### Phase 14: Phase 2 — Additional Providers ✅
- Google provider crate (search + routing initially; expanded to 6 domains in later phases)
- TomTom provider crate (search, routing, traffic, isoline, matching, tour, tiling, imaging)
- MapBox provider crate (search, routing, isoline, matching, tour, tiling, imaging)
- Radar provider crate (search, routing, matching, tour)

### Phase 15: Phase 3 — Architecture Hardening ✅
- **CLI unified dispatch**: `ProviderRegistry` replaces 5 `run_<provider>_commands()` functions. main.rs reduced from ~1750 to ~500 lines.
- **ProviderClient extracted**: Shared HTTP client logic in `everymap-core/src/client/mod.rs`. All 5 provider crates have thin wrappers.
- **Unsupported domain macros**: 7 macros in `everymap-core/src/unsupported.rs`. All 5 provider stubs converted to 1-line macro invocations.
- **Security**: Error body truncation reduced to 256 bytes. Shared `redact_api_key()` from core.
- **Tests**: 624 total (up from 345). ~276 new tests.
- **Clippy**: Zero warnings with `-D warnings`.

### Phase 16: Benchmark Expansion ✅
- `everymap-bench` expanded from 2 domains (geocode, route) to all 10 domains
- `ScenarioParams` enum with typed dispatch across all 10 domains
- 15 predefined scenarios covering all domains
- `BenchProviders` struct holds trait objects for all 10 domains per provider
- Supports 5 providers: HERE, Google, TomTom, MapBox, Radar
- Output formats: table (default), json, markdown

### Phase 17: API Accuracy & Precision Hardening ✅
- **`DepartureTime` enum**: Typed departure time (Now, Timestamp, Iso8601) replacing fragile `Option<String>` across RouteOptions, MatchingOptions, and IsolineOptions. All 5 providers updated with typed matching.
- **Google routing avoids**: Removed invalid `indoor` mapping for DirtRoads and unsupported Tunnels. Unsupported avoid types now filtered with `filter_map`.
- **HERE matching transport modes**: Bus→Bus, Scooter→Motorcycle, Taxi→Taxi (were all incorrectly mapped to Car). Both primary and provider_extra mappings fixed.
- **MapBox reverse geocode radius**: Replaced silent ignore with `log::warn!` diagnostics. MapBox v6 does not support radius/bbox constraints.
- **HERE traffic incidents**: `include_incidents` now actually fetches incidents (was always returning empty vec). Builds bounding box from location+radius, calls incidents API, merges results.
- **CLI Attributes --layer**: Key fixed from singular `"layer"` to plural `"layers"` matching HERE provider's `provider_extra` parsing.
- **Code abbreviations eliminated**: 823+ occurrences fixed (`opts`→`options`, `res`→`response`, `coord`→`coordinate`, `msg`→`message`, `dist`→`total_distance`, `dur`→`total_duration`, `tm`→`transport_mode`, `idx`→`index`).
- **OSS readiness**: Added SECURITY.md, CHANGELOG.md, fixed CODE_OF_CONDUCT.md placeholder, fixed LICENSE year to 2025-2026, removed Cargo.lock from .gitignore (needed for binary crate).
- **Silent parameter drops**: Added `log::warn!` diagnostics for MapBox routing (unsupported `avoid`, `language`), MapBox reverse geocode (unsupported `radius`).

### Phase 18: Public Release Polish ✅
- **Security**: Deleted `.env` with live API keys, removed `tmp/` planning artifacts.
- **Metrics**: Fixed stale counts — 31→35 implementations, 575+→578 tests across all docs. Further expanded to 624 tests in Phase 19.
- **Code abbreviations**: Renamed `here_opts`→`here_options` (256 occurrences, 9 files). Deprecated `wp_dist` → `waypoint_distance`.
- **Logging**: Replaced `eprintln!` with `log::warn!` in MapBox provider (silent parameter drops). Kept `eprintln!` for CLI verbose mode (user-facing).
- **Documentation**: Added `//!` module docs to `everymap-core` (crates.io/docs.rs landing page). Expanded SECURITY.md env var docs. Clarified CONTRIBUTING.md for external contributors.
- **Community files**: Added SUPPORT.md, CODEOWNERS, FUNDING.yml.
- **Cleanup**: Converted commented-out JSON examples to proper doc comments. Removed spurious `#[allow(dead_code)]`.
- **CI**: Added `cli-validate` job (374 CLI assertions) — 9 jobs total.

### Phase 19: Feature & Domain Completion ✅
- **Silent parameter drops**: Added 24 `log::warn!` diagnostics across Google, TomTom, MapBox, Radar, HERE for ignored core fields (avoid, arrival_time, language, limit, radius, heading, departure_time, transport_mode, format, bbox, alternatives).
- **Transport mode collapse warnings**: Added `log::warn!` in all catch-all transport mode mapping arms (Google, TomTom, MapBox routing/isoline/matching).
- **TomTom avoid wiring**: Wired core `AvoidType` → TomTom API params (`avoidTollRoads`, `avoidFerries`, `avoidTunnels`, `avoidMotorways`, `avoidUnpavedRoads`) in both routing and isoline.
- **HERE parameter wiring**: reverse geocode `radius`, imaging `format` from core field, traffic `language` for flow path.
- **Radar parameter wiring**: search `bounding_box` → `near` param, routing `alternatives`+`avoid` from core fields, tour `transport_mode` from core field.
- **TomTom tour**: `TourOptions` no longer ignored — `transport_mode` and `provider_extra` extracted.
- **MapBox**: imaging `format` from core field, tour `provider_extra` passed through.
- **Google geocode**: `limit` post-response truncation.
- **CLI flags**: 20+ new flags — `--language`, `--limit`, `--country`, `--bbox`, `--radius`, `--alternatives`, `--avoid`, `--departure-time`, `--arrival-time`, `--heading`, `--range-type`, `--format`.
- **Integration tests**: Radar HTTP error tests + ext contract tests, bench unit tests, expanded parameter variation tests (Google limit/arrival/avoid, TomTom avoid/transport/bicycle, MapBox departure/heading/format/tour, Radar bbox/alternatives/avoid/transport, HERE radius/parameter-variations).
- **Smoke tests**: Google attributes live smoke, transport variants (bicycle/scooter/bus/taxi), isoline range-type=time, new CLI flags, unsupported domain error paths, verbose mode on 6 commands, MapBox tour expanded, Radar smoke expanded.
- **Test count**: 578 → 624 tests (46 new).
- **Docs**: CLAUDE.md, README.md, PLAN.md updated with new test counts, CLI flags, known/fixed issues.

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 8 |
| Domain traits | 10 |
| CLI commands | 11 |
| Total tests | 624 (nextest) |
| Real implementations | 35 across 5 providers |
| Clippy warnings | 0 |
| Version | 0.2.0 (0.2.1 upcoming) |

---

## Verification & Quality Gates
- **TDD**: 624 tests (unit + contract + CLI integration + error cases + bench), all passing with nextest
- **SOLID**: `everymap-core` has zero knowledge of any provider crate
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green
- **Typed time**: `DepartureTime` enum eliminates string-guessing across all departure/arrival time fields
- **No abbreviations**: All code uses full descriptive variable names (`here_options`, `waypoint_distance`).

---

## Future Work
- [ ] OAuth2 auth provider implementation
- [ ] Upgrade Google routing from legacy Directions API to Routes API v2
- [ ] Add `Moderate` variant to core `IncidentSeverity` (TomTom traffic)
- [x] Wire `avoid` parameter for TomTom routing (API supports it) — wired in both routing and isoline
- [x] Add missing CLI flags: `--language`, `--limit`, `--avoid`, `--radius`, `--arrival-time`, `--alternatives` — all 20+ flags added
- [ ] Configurable image size for `map-image` command
- [x] CI/CD pipeline (GitHub Actions — 9 jobs: fmt, clippy, docs, audit, MSRV, test, build, check-publish, cli-validate)
- [x] Config file permission check (`~/.everymap/config.toml` world-readable warning) — Unix-only, warning on stderr
- [x] Zeroize API keys in memory (`zeroize` crate) — `#[zeroize(drop)]` on ApiKeyProvider and HeaderAuthProvider
- [x] Avoid redundant serialization in CLI — `write_output()` uses `serde_json::to_writer` for direct stdout write
- [x] OSS documentation: SECURITY.md, CHANGELOG.md, CODE_OF_CONDUCT.md contact method