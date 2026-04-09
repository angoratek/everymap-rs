# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, MapBox, TomTom, Google) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Address`, `Polyline`, `FlexiblePolyline`.
    - Domain Traits: Interfaces for all 10 domains, each with associated `Options` and `Response` types.
    - Enriched Response Types: `SearchResult`, `RouteResult`, `TrafficFlow`, `TrafficIncident`, `IsolineResult`, `MatchedPoint`, `TourStop`, etc.
    - Auth Traits: `AuthProvider` interface (`ApiKeyProvider`, future `OAuth2Provider`).
    - Error System: Structured `EveryMapError` with `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, etc.
- **`everymap-providers-here`**: The HERE Technologies implementation.
    - Maps OpenAPI specs to Rust types with full coverage.
    - Implements `everymap-core` traits.
    - Each domain has its own module with rich provider-specific types.
    - Shared `HereLatLng` in `domain/geo.rs` (eliminates duplication).
    - `From<HereX> for CoreX` conversions for type-safe provider → core mapping.
- **`everymap-providers-mapbox` / `tomtom` / `google`**: (Future) Implementation crates.
- **`everymap-cli`**: CLI tool for interacting with providers.
    - `clap` derive macros, `tokio` runtime, JSON output.
    - Commands: `geocode`, `reverse-geocode`, `route`, `traffic`, `position`, `isoline`.
    - Auth via `--api-key` or `EVERYMAP_API_KEY` env var.

### 2. Domain Abstraction (Middleware)
Each API is treated as a "Provider" of a "Domain".

| Domain | Core Trait | HERE Implementation | Tests |
| :--- | :--- | :--- | :--- |
| **Search** | `Geocoder` | `HereGeocoder` | 5 |
| **Routing** | `Router` | `HereRouter` | 1 |
| **Isolines** | `IsolineProvider` | `HereIsoline` | 2 |
| **Tracing** | `RouteMatcher` | `HereRouteMatcher` | 2 |
| **Logistics** | `TourPlanner` | `HereTourPlanner` | 4 |
| **Traffic** | `TrafficProvider` | `HereTraffic` | 3 |
| **Tiling** | `TileProvider` | `HereTileProvider` | 3 |
| **Positioning** | `NetworkPositioner` | `HerePositioner` | 3 |
| **Attributes** | `AttributeProvider` | `HereAttributeProvider` | 2 |
| **Imaging** | `MapImageProvider` | `HereMapImageProvider` | 2 |

### 3. Authentication Layer
- **`AuthProvider` Trait**: `async fn apply(&self, builder: RequestBuilder) -> Result<RequestBuilder>`
- **`ApiKeyProvider`**: Implements `AuthProvider` (injects API key as query param).
- **`OAuth2Auth`**: (Future) Implements `AuthProvider` with token caching/refresh logic.

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
- **Enriched core response types**: `SearchResult` now has structured `Address`, `SearchResultType` enum, `confidence`, `categories`, `bounding_box`, `raw` escape hatch
- **RouteResponse** now has `routes: Vec<RouteResult>` with `transport_mode`, `steps`, `bounding_box`
- **TrafficResponse** now has `flows: Vec<TrafficFlow>` and `incidents: Vec<TrafficIncident>`
- **IsolineResponse** now has `isolines: Vec<IsolineResult>` with `range` field
- **TraceResponse** now has `matched_points: Vec<MatchedPoint>` with `confidence` and `road_name`
- **TourResponse** now has `stops: Vec<TourStop>`, `total_distance`, `total_duration`, `unassigned_count`
- **Response type consistency**: Positioning now returns core `PositioningResponse` (not HERE-specific type)
- **`From` trait conversions**: `From<HereAddress> for Address`, `From<HereSearchItem> for SearchResult`
- **Shared geo types**: `HereLatLng` extracted to `domain/geo.rs` (no more duplication across search/routing)
- **Enriched error types**: `EveryMapError` now has `HttpError { status, message, body }`, `AuthError { provider, message }`, `ProviderError { provider, code, message }`, `RateLimited { provider, retry_after_secs }`, plus helper methods `is_rate_limited()`, `is_auth_error()`, `is_status()`
- **`everymap-cli` Phase 1**: `clap`-based CLI with `geocode`, `reverse-geocode`, `route`, `traffic`, `position`, `isoline` commands

---

## Verification & Quality Gates
- **TDD**: 31 contract tests using `wiremock`, all passing
- **SOLID**: `everymap-core` has zero knowledge of `everymap-providers-here`
- **Lightweight**: No unnecessary dependencies leaked into core
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green

### Remaining Work
- [ ] Injectable HTTP client trait (for custom timeout/pool/proxy configuration)
- [ ] Reduce parameter serialization boilerplate (helper trait or derive macro)
- [ ] everymap-cli Phase 2 (config file, table output, all domains)
- [ ] Extension traits for provider-specific methods (e.g., `HereGeocoderExt`)
- [ ] everymap-providers-google (search + routing)
- [ ] OAuth2 auth provider implementation
- [ ] Dynamic provider registry for runtime dispatch
- [ ] Future provider crates (MapBox, TomTom, Google Maps)
- [ ] Benchmarking with `criterion` for large response deserialization
- [ ] Comprehensive `rustdoc` examples for each domain
- [ ] CI/CD pipeline (GitHub Actions with clippy, fmt, nextest)