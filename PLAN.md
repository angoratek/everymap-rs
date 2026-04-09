# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, MapBox, TomTom, Google) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Point`, `Polyline`, `FlexiblePolyline`.
    - Domain Traits: Interfaces for all 10 domains, each with associated `Options` and `Response` types.
    - Auth Traits: `AuthProvider` interface (`ApiKeyProvider`, future `OAuth2Provider`).
    - Error System: Unified `EveryMapError` using `thiserror`.
- **`everymap-providers-here`**: The HERE Technologies implementation.
    - Maps OpenAPI specs to Rust types with full coverage.
    - Implements `everymap-core` traits.
    - Each domain has its own module with rich provider-specific types.
- **`everymap-providers-mapbox` / `tomtom` / `google`**: (Future) Implementation crates.

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

## Implementation Status: ALL 10 DOMAINS COMPLETE

### Phase 0: Architecture Foundations ✅
- Per-domain base URLs (HereClient simplified, each domain has const BASE_URL)
- FlexiblePolyline decoder (using `flexpolyline` crate, full encode/decode)
- Core trait redesign (all traits have associated Response + Options types)
- Domain mod.rs reorganized to avoid glob conflicts

### Phase 1: Search Domain ✅
- Geocode + reverseGeocode with full params (18+ query params)
- Discover endpoint with full options
- Autosuggest endpoint with full options
- Rich response types (HereSearchItem, HereAddress, HereCategory, etc.)
- 5 contract tests passing

### Phase 2: Routing Domain ✅
- Full parameter set (transport_mode, routing_mode, alternatives, via, avoid, exclude, etc.)
- Rich response types (HereRouteApiResponse, HereRouteSection, HereRouteAction, etc.)
- Truck/EV/Scooter/Taxi parameter structs
- 1 contract test passing

### Phase 3: Isoline Domain ✅
- Full parameter set (range_type, transport_mode, routing_mode, optimize_for, avoid, etc.)
- Rich response types (HereIsolineApiResponse, HereIsoline, HerePolygon, etc.)
- Correct polyline format (outer/holes structure)
- 2 contract tests passing

### Phase 4: Traffic Domain ✅
- Flow endpoint with full params (in, locationReferencing, minJamFactor, etc.)
- Incidents endpoint with full params (criticality, type, lang, units, etc.)
- Rich response types (HereFlowResponse, HereCurrentFlow, HereIncidentsResponse, etc.)
- Core trait delegates to rich `get_flow()` method
- 3 contract tests passing

### Phase 5: Matching Domain ✅
- Full 60+ parameters organized into groups (match mode, matching params, vehicle, emission, restrictions, commercial, time, response attrs, toll, advanced)
- Rich response types (HereMatchApiResponse, HereMatchedPoint, HereMatchedRoute, HereMatchedLeg, etc.)
- Parameter enums (MatchMode, LegalConstraint, TrailerType, EmissionType, etc.)
- 2 contract tests passing

### Phase 6: Tour Planning Domain ✅
- Full typed request/response (TourProblem, Fleet, VehicleType, Plan, Job, etc.)
- Async endpoints (solve_async, get_async_status, get_solution, cancel)
- Rich response types (TourSolution, TourStatistic, TourTour, TourStop, etc.)
- All enum types (Profile, Objective, RelationType, ActivityType, etc.)
- 4 contract tests passing

### Phase 7: Tiling Domain ✅
- Vector Tile API v2 with layer selection (Mapbox, Base, Core, Hybrid)
- Binary tile response with content-type handling
- Format selection (OmnichannelVector, Protobuf)
- Political view parameter
- 3 contract tests passing

### Phase 8: Positioning Domain ✅
- Network Positioning API v2 with WLAN, cell, Bluetooth observations
- Rich request types (WlanAccessPoint, CellTower, BluetoothBeacon, RadioType)
- Rich response types (PositioningResponse, PositionLocation, PositionAltitude)
- Fallback behavior support
- 3 contract tests passing

### Phase 9: Attributes Domain ✅
- Map Attributes API v8 with layer selection (Roads, AdminAreas, Buildings, etc.)
- Query by bbox, IDs, include/exclude fields
- Format selection (Json, GeoJson, Protobuf)
- Language and political view parameters
- 2 contract tests passing

### Phase 10: Imaging Domain ✅
- Map Image API v3 with center/zoom/size
- Format selection (PNG, JPG, GIF, BMP, SVG, PNG8, PNG32)
- Style selection, language, POI, overlay, background color
- Binary image response with content-type handling
- 2 contract tests passing

---

## Verification & Quality Gates

- **TDD**: 31 contract tests using `wiremock`, all passing
- **SOLID**: `everymap-core` has zero knowledge of `everymap-providers-here`
- **Lightweight**: No unnecessary dependencies leaked into core
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green

### Remaining Work (Phase 3 from original plan)
- [ ] Benchmarking with `criterion` for large response deserialization
- [ ] Comprehensive `rustdoc` examples for each domain
- [ ] CI/CD pipeline (GitHub Actions with clippy, fmt, nextest)
- [ ] OAuth2 auth provider implementation
- [ ] Future provider crates (MapBox, TomTom, Google)