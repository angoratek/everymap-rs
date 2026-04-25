# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, Google, TomTom, MapBox, Radar) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace (8 crates)

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Address`, `Polyline`, `FlexiblePolyline`.
    - 13 Domain Traits: `Geocoder`, `Router`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `TrafficProvider`, `TileProvider`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`, `GeofenceProvider`, `TripTracker`, `FraudDetector`.
    - Core Options Types: `GeocodeOptions`, `RouteOptions`, etc. with `provider_extra: Option<serde_json::Value>` escape hatch.
    - Enriched Response Types: `SearchResult`, `RouteResult`, `TrafficFlow`, `TrafficIncident`, `IsolineResult`, `MatchedPoint`, `TourResponse`, etc.
    - Auth Traits: `AuthProvider` (`ApiKeyProvider`, `HeaderAuthProvider`, future `OAuth2Provider`).
    - Error System: Structured `EveryMapError` with `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
    - `ProviderClient`: Consolidated HTTP client logic (request, request_json, post_json, redact_api_key, truncate_str).
    - 10 Unsupported Domain Macros: `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_tile!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`, `unsupported_geofence!`, `unsupported_trip_tracker!`, `unsupported_fraud_detector!`.
- **`everymap-providers-here`**: HERE Technologies — 10 domains implemented (all common domains).
- **`everymap-providers-google`**: Google Maps — 6 domains implemented (search, routing, matching, positioning, attributes, imaging).
- **`everymap-providers-tomtom`**: TomTom — 8 domains implemented (search, routing, traffic, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-mapbox`**: MapBox — 7 domains implemented (search, routing, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-radar`**: Radar — 7 domains (search, routing, matching, tour, geofencing, tracking, fraud). Geofencing/tracking/fraud are Radar-exclusive.
- **`everymap-cli`**: CLI tool with 19 commands, unified `ProviderRegistry` dispatch.
- **`everymap-bench`**: Cross-provider benchmark framework covering all 13 domains with typed `ScenarioParams` dispatch.

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
| **Geofencing** | `GeofenceProvider` | N/A | N/A | N/A | N/A | ✅ |
| **Tracking** | `TripTracker` | N/A | N/A | N/A | N/A | ✅ |
| **Fraud** | `FraudDetector` | N/A | N/A | N/A | N/A | ✅ |

**38 real implementations** across 5 providers. 15 stubs. 12 N/A (provider-exclusive domains).

### 3. Key Design Decisions

- **Concrete options types** (not associated types): Enables `Box<dyn Geocoder>` dynamic dispatch.
- **Core response types are rich enough for 80% of use cases**: All fields are `Option<T>`. `raw: Option<serde_json::Value>` escape hatch for the remaining 20%.
- **Extension traits** for provider-specific methods: `HereGeocoderExt::discover()`, etc.
- **Feature flags** for providers in CLI: `--features here,google` to control compile time.
- **Version independently**: Each crate follows its own semver pace.
- **ProviderClient consolidation**: All 5 provider clients delegate to shared `ProviderClient` from core.
- **Unsupported domain macros**: 10 macros eliminate boilerplate stubs.
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
- Google provider crate (search + routing)
- TomTom provider crate (search, routing, traffic, isoline, matching, tour, tiling, imaging)
- MapBox provider crate (search, routing, isoline, matching, tour, tiling, imaging)
- Radar provider crate (search, routing, matching, tour, geofencing, tracking, fraud)

### Phase 15: Phase 3 — Architecture Hardening ✅
- **CLI unified dispatch**: `ProviderRegistry` replaces 5 `run_<provider>_commands()` functions. main.rs reduced from ~1750 to ~500 lines.
- **ProviderClient extracted**: Shared HTTP client logic in `everymap-core/src/client/mod.rs`. All 5 provider crates have thin wrappers.
- **Unsupported domain macros**: 10 macros in `everymap-core/src/unsupported.rs`. All 5 provider stubs converted to 1-line macro invocations.
- **Security**: Error body truncation reduced to 256 bytes. Shared `redact_api_key()` from core.
- **Tests**: 540 total (up from 345). ~180 new core domain tests.
- **Clippy**: Zero warnings with `-D warnings`.

### Phase 16: Benchmark Expansion ✅
- `everymap-bench` expanded from 2 domains (geocode, route) to all 13 domains
- `ScenarioParams` enum with 14 variants for typed dispatch
- 15 predefined scenarios covering all domains
- `BenchProviders` struct holds trait objects for all 13 domains per provider
- Supports 5 providers: HERE, Google, TomTom, MapBox, Radar
- Output formats: table (default), json, markdown

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 8 |
| Domain traits | 13 |
| CLI commands | 19 |
| Total tests | 747 (nextest) |
| Real implementations | 38 across 5 providers |
| Clippy warnings | 0 |
| Version | 0.2.0 |

---

## Verification & Quality Gates
- **TDD**: 747 tests (unit + contract + CLI integration + error cases), all passing with nextest
- **SOLID**: `everymap-core` has zero knowledge of any provider crate
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green

---

## Future Work
- [ ] OAuth2 auth provider implementation
- [x] CI/CD pipeline (GitHub Actions with clippy, fmt, nextest) — 3 parallel jobs, nextest via taiki-e/install-action
- [x] Config file permission check (`~/.everymap/config.toml` world-readable warning) — Unix-only, warning on stderr
- [x] Zeroize API keys in memory (`zeroize` crate) — `#[zeroize(drop)]` on ApiKeyProvider and HeaderAuthProvider
- [x] Avoid redundant serialization in CLI — `write_output()` uses `serde_json::to_writer` for direct stdout write