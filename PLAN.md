# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, MapBox, TomTom, Google) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Address`, `Polyline`, `FlexiblePolyline`.
    - Domain Traits: Interfaces for all 10 domains, returning concrete response types.
    - Core Options Types: `GeocodeOptions`, `RouteOptions`, etc. with `provider_extra: Option<serde_json::Value>` escape hatch.
    - Enriched Response Types: `SearchResult`, `RouteResult`, `TrafficFlow`, `TrafficIncident`, `IsolineResult`, `MatchedPoint`, `TourStop`, etc.
    - Auth Traits: `AuthProvider` interface (`ApiKeyProvider`, future `OAuth2Provider`).
    - Error System: Structured `EveryMapError` with `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
- **`everymap-providers-here`**: The HERE Technologies implementation.
    - Maps OpenAPI specs to Rust types with full coverage.
    - Implements `everymap-core` traits.
    - Each domain has its own module with rich provider-specific types.
    - Shared `HereLatLng` in `domain/geo.rs` (eliminates duplication).
    - `From<HereX> for CoreX` conversions for type-safe provider → core mapping.
- **`everymap-providers-google`**: Google Maps implementation.
    - Geocoder ← Geocoding API (search + reverse geocode)
    - Router ← Directions API (route calculation with Google polyline decoding)
    - Unsupported domains return `UnsupportedDomain` error (Traffic, Isoline, Tour, Attributes, Tiling, Positioning, Matching, Imaging)
    - `GoogleClient` with auth handling and error status checking
    - `From<GoogleGeocodeResult> for SearchResult` and `From<GoogleRoute> for RouteResult` conversions
- **`everymap-providers-mapbox` / `tomtom`** (future): Implementation crates.
- **`everymap-cli`**: CLI tool for interacting with providers.
    - `clap` derive macros, `tokio` runtime, JSON output.
    - Commands: `geocode`, `reverse-geocode`, `route`, `traffic`, `position`, `isoline`, plus planned: `match-route`, `tour`, `tile`, `attributes`, `map-image`.
    - Auth via `--api-key` or `EVERYMAP_API_KEY` env var or `~/.everymap/config.toml`
    - Multi-provider support: `--provider here` (default) or `--provider google`
    - `Box<dyn Trait>` dispatch for multi-provider support

### 2. Domain Abstraction (Middleware)
Each API is treated as a "Provider" of a "Domain".

| Domain | Core Trait | HERE Implementation | Google Implementation |
| :--- | :--- | :--- | :--- |
| **Search** | `Geocoder` | `HereGeocoder` | `GoogleGeocoder` ✅ |
| **Routing** | `Router` | `HereRouter` | `GoogleRouter` ✅ |
| **Isolines** | `IsolineProvider` | `HereIsoline` | `GoogleIsoline` (stub) |
| **Tracing** | `RouteMatcher` | `HereRouteMatcher` | `GoogleRouteMatcher` (stub) |
| **Logistics** | `TourPlanner` | `HereTourPlanner` | `GoogleTourPlanner` (stub) |
| **Traffic** | `TrafficProvider` | `HereTraffic` | `GoogleTraffic` (stub) |
| **Tiling** | `TileProvider` | `HereTileProvider` | `GoogleTileProvider` (stub) |
| **Positioning** | `NetworkPositioner` | `HerePositioner` | `GooglePositioner` (stub) |
| **Attributes** | `AttributeProvider` | `HereAttributeProvider` | `GoogleAttributeProvider` (stub) |
| **Imaging** | `MapImageProvider` | `HereMapImageProvider` | `GoogleMapImageProvider` (stub) |

### 3. Key Design Decisions

- **Concrete options types** (not associated types): `GeocodeOptions`, `RouteOptions`, etc. live in core with `provider_extra: Option<serde_json::Value>` for provider-specific params. This enables `Box<dyn Geocoder>` dynamic dispatch.
- **Core response types are rich enough for 80% of use cases**: All fields are `Option<T>` where any provider may not provide data. `raw: Option<serde_json::Value>` escape hatch for the remaining 20%.
- **Extension traits** for provider-specific methods: `HereGeocoderExt::discover()`, `GoogleGeocoderExt::place_search()`.
- **Feature flags** for providers in CLI: `--features here,google` to control compile time.
- **Version independently**: Each crate follows its own semver pace.

### 4. Authentication Layer
- **`AuthProvider` Trait**: `async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder>`
- **`ApiKeyProvider`**: Implements `AuthProvider` (injects API key as query param).
- **`OAuth2Auth`** (planned): Implements `AuthProvider` with token caching/refresh logic.

---

## Implementation Status

### Phase 0: Architecture Foundations ✅
- Per-domain base URLs (HereClient simplified, each domain has const BASE_URL)
- FlexiblePolyline decoder (using `flexpolyline` crate, full encode/decode)
- Core trait redesign (all traits have associated Response + Options types)
- Domain mod.rs reorganized to avoid glob conflicts

### Phase 1-10: All 10 HERE Domains ✅
- All domains implemented with full parameter coverage
- 31 contract tests passing
- Rich HERE-specific types alongside core trait impls

### Phase 11: Multi-Provider Architecture Improvements ✅
- Enriched core response types with structured fields + `raw` escape hatch
- Response type consistency (all domains return core types, including positioning)
- `From` trait conversions for search (`HereAddress→Address`, `HereSearchItem→SearchResult`, `HereLatLng→Coordinate`) and traffic (`HereFlowItem→TrafficFlow`, `HereIncident→TrafficIncident`)
- Shared geo types: `HereLatLng` centralized in `domain/geo.rs`, `HereIsolineLatLng` duplication removed
- Enriched error types: `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `ValidationError`, `SerializationError`, `UnsupportedDomain`
- `everymap-cli` Phase 1: 6 commands working (geocode, reverse-geocode, route, traffic, position, isoline)

### Phase 12: Concrete Options Refactor ✅
- [x] Create core options types (`GeocodeOptions`, `RouteOptions`, `TrafficOptions`, `IsolineOptions`, `PositioningOptions`, `MatchingOptions`, `TourOptions`, `TileOptions`, `AttributeOptions`, `ImageOptions`)
- [x] Migrate all 10 traits from `type Options` associated types to concrete parameter types
- [x] Remove request wrappers (`GeocodeRequest<O>`, `RouteRequest<O>`, etc.)
- [x] Migrate HERE provider to use core options with `*_from_core()` conversion helpers
- [x] Migrate CLI to use core option types directly
- [x] Add `UnsupportedDomain` error variant
- [x] Add `HttpClient` trait + `DefaultHttpClient` to core
- [x] Add `From<HereFlowItem> for TrafficFlow`, `From<HereIncident> for TrafficIncident` conversions
- [x] Remove `HereIsolineLatLng` duplication (consolidated to `HereLatLng`)
- [x] Add extension traits: `HereGeocoderExt`, `HereTrafficExt`, `HerePositionerExt`, `HereTourPlannerExt`
- [x] Complete CLI domain coverage: all 11 commands (geocode, reverse-geocode, route, traffic, position, isoline, match-route, tour, tile, attributes, map-image)
- [x] All 31 contract tests pass, clippy clean
- [x] 28 unit tests in `everymap-core` (options, types, error)
- [x] CLI output formats: `--output json|pretty|summary`
- [x] Version bumped to 0.2.0

### Phase 13: From Conversions & Testing ✅
- [x] Add `From` trait conversions for all domain types (routing, isoline, positioning, matching, tour)
- [x] `From<HereRoute> for RouteResult` and `From<HereRouteSection> for RouteResult`
- [x] `From<HereIsolineLegacy> for IsolineResult`
- [x] `From<PositioningResponse> for CorePositioningResponse`
- [x] `From<HereMatchedPoint> for MatchedPoint`
- [x] `From<TourSolution> for TourResponse`
- [x] 52 unit tests in `everymap-core` (all 10 domains + error + types)
- [x] 31 contract tests + 5 error case tests
- [x] CLI config file support (`~/.everymap/config.toml`)
- [x] HTTP error handling in HereClient (401, 403, 404, 429, 500)

See `tmp/multi-provider-architecture-plan.md` for full details.

---

## Verification & Quality Gates
- **TDD**: 31 contract tests + 5 error case tests + 52 unit tests (88 total), all passing
- **SOLID**: `everymap-core` has zero knowledge of `everymap-providers-here`
- **Lightweight**: No unnecessary dependencies leaked into core
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green

### Future Work
- [x] Google provider crate (search + routing)
- [ ] OAuth2 auth provider implementation
- [x] CLI config file support (`~/.everymap/config.toml`)
- [x] CLI output formats (pretty, summary)
- [ ] CLI output format: table
- [ ] Benchmarking with `criterion` for large response deserialization
- [ ] CI/CD pipeline (GitHub Actions with clippy, fmt, nextest)