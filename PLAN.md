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
- **Shared workspace version**: All crates version together via `[workspace.package]` (single-cadence releases).
- **ProviderClient consolidation**: All 5 provider clients delegate to shared `ProviderClient` from core.
- **Unsupported domain macros**: 7 macros eliminate boilerplate stubs.
- **Unified CLI dispatch**: `ProviderRegistry` replaces 5 duplicated command handler functions.

### 4. Authentication Layer
- **`AuthProvider` Trait**: `async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder>`
- **`ApiKeyProvider`**: Injects API key as query param (HERE, Google, TomTom, MapBox).
- **`HeaderAuthProvider`**: Injects API key as Authorization header (Radar).
- **`OAuth2Auth`** (planned): Token caching/refresh logic.

---

## Completed Highlights

The workspace was built across 20 implementation phases (Phases 0–19), all complete:

- **Foundations**: per-domain base URLs, FlexiblePolyline decoder, core trait redesign, concrete Options refactor, `From` conversions, `HttpClient` trait.
- **Multi-provider architecture**: all 5 provider crates (HERE 10 domains, Google 6, TomTom 8, MapBox 7, Radar 4), shared geo types, enriched errors.
- **Architecture hardening**: unified CLI `ProviderRegistry` dispatch, `ProviderClient` extraction, 7 unsupported-domain macros, error-body truncation, shared key redaction.
- **API accuracy**: typed `DepartureTime` enum across all providers, silent-parameter-drop warnings (24 locations), TomTom avoid wiring, HERE parameter wiring (reverse radius, imaging format, traffic language), Radar parameter wiring (search bbox, tour transport mode).
- **Benchmark expansion**: all 10 domains, typed `ScenarioParams` dispatch, 15 scenarios, table/json/markdown output.
- **OSS readiness**: community files (SECURITY.md, CHANGELOG.md, CODE_OF_CONDUCT.md, SUPPORT.md, CODEOWNERS, FUNDING.yml), CLI validation framework (374 assertions), 9-job CI pipeline, tag-triggered release workflow, MSRV 1.86, cargo-deny audit, reqwest 0.13 migration.

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 8 |
| Domain traits | 10 |
| CLI commands | 11 |
| Total tests | 623 run, 9 skipped (nextest) |
| Real implementations | 35 across 5 providers |
| Clippy warnings | 0 |
| MSRV | 1.86 |
| Version | 0.2.1 |

---

## Verification & Quality Gates
- **TDD**: 623 tests (unit + contract + CLI integration + error cases + bench), all passing with nextest
- **SOLID**: `everymap-core` has zero knowledge of any provider crate
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green
- **Typed time**: `DepartureTime` enum eliminates string-guessing across all departure/arrival time fields
- **No abbreviations**: All code uses full descriptive variable names (`here_options`, `waypoint_distance`).

---

## Roadmap

- [ ] OAuth2 auth provider implementation
- [ ] Upgrade Google routing from legacy Directions API to Routes API v2
- [ ] Add `Moderate` variant to core `IncidentSeverity` (TomTom traffic)
- [ ] Configurable image size for `map-image` command
- [ ] Wire core `avoid`/`alternatives` fields to Radar routing API (currently only via `provider_extra`)
- [ ] Provider client macro to reduce boilerplate across crates
- [ ] Compile-time feature-gated providers in CLI to reduce binary size
- [x] First crates.io release (v0.2.1): publish core + 5 providers + cli via `scripts/publish-crates.sh` / `release.yml`