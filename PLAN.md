# Project Plan: EveryMap-RS (State-of-the-art Geospatial API Wrapper)

## Vision
To build the most robust, type-safe, and modular Rust ecosystem for geospatial services. The architecture allows consumers to switch between providers (HERE, Google, TomTom, MapBox, Radar) with zero changes to business logic, utilizing a domain-driven middleware abstraction.

---

## Architectural Blueprint

### 1. Crate Strategy: Modular Workspace (13 crates)

- **`everymap-core`**: The bedrock. Zero-dependency where possible.
    - Shared Types: `Coordinate`, `BoundingBox`, `Address`, `Polyline`, `FlexiblePolyline`.
    - Contracts: the ten original domain traits (`Geocoder`, `Router`, `IsolineProvider`, `RouteMatcher`, `TourPlanner`, `TrafficProvider`, `TileProvider`, `NetworkPositioner`, `AttributeProvider`, `MapImageProvider`) plus navigation-grade `Router`, typed `RasterTileProvider`/`VectorTileProvider`, `PlaceSearchProvider`, `AutocompleteProvider`, `DistanceMatrixProvider`, `ElevationProvider` and `OfflineRoutingProvider`.
    - Core Options Types: `GeocodeOptions`, `RouteOptions`, `TileSourcePacing`, etc. with `provider_extra: Option<serde_json::Value>` escape hatch.
    - Enriched Response Types: `SearchResult`, `RouteResult` (legs, maneuvers, lanes), `TrafficFlow`, `TrafficIncident`, `DistanceMatrixResponse`, `AutocompleteResponse`, typed tiles, etc.
    - Auth Traits: `AuthProvider` (`ApiKeyProvider`, `HeaderAuthProvider`, `OAuth2Provider`, `NoAuthProvider`).
    - Error System: Structured `EveryMapError` with `HttpError`, `AuthError`, `ProviderError`, `RateLimited`, `UnsupportedDomain`, etc.
    - `ProviderClient`: Consolidated request logic — transport seam (`HttpTransport`: reqwest natively, `fetch` in the browser), pacing, bounded retries, timeouts, redacting observers, and a `web-time` clock so the same code runs on every target.
    - Unsupported Domain Macros: `unsupported_isoline!`, `unsupported_traffic!`, `unsupported_tour!`, `unsupported_tile!`, `unsupported_positioner!`, `unsupported_attributes!`, `unsupported_image!`.
- **`everymap-providers-here`**: HERE Technologies — 10 domains implemented (all common domains).
- **`everymap-providers-google`**: Google Maps — 6 domains implemented (search, routing, matching, positioning, attributes, imaging).
- **`everymap-providers-tomtom`**: TomTom — 8 domains implemented (search, routing, traffic, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-mapbox`**: MapBox — 7 domains implemented (search, routing, isoline, matching, tour, tiling, imaging).
- **`everymap-providers-radar`**: Radar — 4 domains (search, routing, matching, tour).
- **`everymap-providers-mapquest`**: MapQuest — geocoding, place search, autocomplete, navigation-grade routing, distance matrix, traffic, raster tiles.
- **`everymap-providers-osrm`**, **`everymap-providers-valhalla`**: keyless routing engines over a deployment address.
- **`everymap-providers-url-template-raster`**, **`-vector`**: any `{z}/{x}/{y}` tileset, with named OSM US and OpenFreeMap vector sources.
- **`everymap-cli`**: CLI tool with 11 commands, unified `ProviderRegistry` dispatch.
- **`everymap-bench`**: Cross-provider benchmark framework with typed `ScenarioParams` dispatch.

Every provider crate builds for `wasm32-unknown-unknown` and reaches the network only through the shared client; the CLI and the bench stay native.

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
- **`OAuth2Provider`**: Client-credentials flow with cached access tokens (expiry-based refresh) and `Authorization: Bearer` injection.

---

## Completed Highlights

The workspace was built across 20 implementation phases (Phases 0–19), all complete:

- **Foundations**: per-domain base URLs, FlexiblePolyline decoder, core trait redesign, concrete Options refactor, `From` conversions, `HttpClient` trait.
- **Multi-provider architecture**: all 5 provider crates (HERE 10 domains, Google 6, TomTom 8, MapBox 7, Radar 4), shared geo types, enriched errors.
- **Architecture hardening**: unified CLI `ProviderRegistry` dispatch, `ProviderClient` extraction, 7 unsupported-domain macros, error-body truncation, shared key redaction.
- **API accuracy**: typed `DepartureTime` enum across all providers, silent-parameter-drop warnings (24 locations), TomTom avoid wiring, HERE parameter wiring (reverse radius, imaging format, traffic language), Radar parameter wiring (search bbox, tour transport mode).
- **Benchmark expansion**: all 10 domains, typed `ScenarioParams` dispatch, 15 scenarios, table/json/markdown output.
- **OSS readiness**: community files (SECURITY.md, CHANGELOG.md, CODE_OF_CONDUCT.md, SUPPORT.md, CODEOWNERS, FUNDING.yml), CLI validation framework (374 assertions), 9-job CI pipeline, tag-triggered release workflow, MSRV 1.86, cargo-deny audit, reqwest 0.13 migration.
- **Roadmap completion**: OAuth2 client-credentials provider, Google Routes API v2 migration, `Moderate` severity variant, configurable image size, Radar avoid/alternatives wiring, `provider_client!` macro.

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 13 (core, 11 provider crates, CLI, bench) |
| Contracts | The 10 original domain traits plus navigation-grade `Router`, typed `RasterTileProvider`/`VectorTileProvider`, `PlaceSearchProvider`, `AutocompleteProvider`, `DistanceMatrixProvider`, `ElevationProvider` and `OfflineRoutingProvider` |
| Browser-ready crates | `everymap-core`, the MapQuest, OSRM, Valhalla and both URL-template providers |
| Total tests | 97 suites green on 0.3.1 |
| Clippy warnings | 0 |
| MSRV | 1.86 |
| Version | 0.3.1 |

---

## Verification & Quality Gates
- **TDD**: 654 tests (unit + contract + CLI integration + error cases + bench), all passing with nextest
- **SOLID**: `everymap-core` has zero knowledge of any provider crate
- **Clippy**: `cargo clippy -- -D warnings` clean
- **Tests**: `cargo test` all green
- **Typed time**: `DepartureTime` enum eliminates string-guessing across all departure/arrival time fields
- **No abbreviations**: All code uses full descriptive variable names (`here_options`, `waypoint_distance`).

---

## Roadmap

- [x] OAuth2 auth provider implementation
- [x] Upgrade Google routing from legacy Directions API to Routes API v2
- [x] Add `Moderate` variant to core `IncidentSeverity` (TomTom traffic)
- [x] Configurable image size for `map-image` command
- [x] Wire core `avoid`/`alternatives` fields to Radar routing API (currently only via `provider_extra`)
- [x] Provider client macro to reduce boilerplate across crates
- [ ] Compile-time feature-gated providers in CLI to reduce binary size

---

### M1 — Documentation and tooling parity · *status: open (owner-directed 2026-11-04)*

> Verify: every page that names the provider set, the crate list, the contract list, the test count or the version matches the workspace; `scripts/generate-readmes.sh --check`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` and the README link check are green; and `everymap-cli` and `everymap-bench` can select every provider the workspace ships.

The 0.2.2-era prose still describes five providers, ten traits and eight crates. The workspace now carries thirteen crates, four more contracts and a browser target, so the documents are the largest remaining source of wrong answers — for people and for agents alike.

- [x] `README.md`: the headline, provider list, workspace tree, domain matrix, crate table, installation snippet, feature list and contribution guide name all thirteen crates and every contract; add the browser transport (and that the five original providers, the CLI and the bench are native-only), the `web-time` clock, the MapQuest/OSRM/Valhalla/URL-template crates, and the 0.3.1 fixes.
- [x] `PLAN.md`: refresh the Architectural Blueprint (crate count, contracts, the provider matrix), the Completed Highlights and the metrics from the workspace rather than from memory.
- [x] `CLAUDE.md`: the repository guide's architecture, commands, wasm closure and release procedure reflect the current workspace.
- [x] `TESTING.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md` (round 194): provider lists, live-test instructions (including `EVERYMAP_MAPQUEST_API_KEY`), the browser matrix, and the credential/transport facts a contributor needs.
- [x] `docs/readme/*.md` and the generated `<crate>/README.md`: each crate body lists the contracts it implements and its platform support; regenerate with `scripts/generate-readmes.sh`.
- [x] `everymap-cli` (round 196): `--provider mapquest|osrm|valhalla` (MapQuest for every contract it serves; OSRM and Valhalla for routing, with the deployment URL as the credential), the URL-template raster and vector sources where a command fits, `~/.everymap/config.toml` sections and env names for each, and the unsupported-capability answer where a provider has no implementation; update `docs/readme/everymap-cli.md`, `scripts/smoke-test-cli.sh` and `scripts/validate-cli.sh`.
- [x] `everymap-bench` (round 196): benchmark MapQuest, OSRM and Valhalla beside the five original providers through the same typed scenario dispatch, and document how to run them.
- [ ] `docs/rfc/0001-transport-seam-and-wasm.md`: record phases 3 and 4 as landed, and add 0.3.1.
- [x] CI (round 199): the docs job fails on a rustdoc warning, on stale generated READMEs (`scripts/generate-readmes.sh --check`) and on a CLI that no longer builds or answers `--help`.

### M2 — `everymap-providers-photon` · *status: open (owner-directed 2026-11-04)*

> Verify: a provider crate implements `Geocoder`, `PlaceSearchProvider` and `AutocompleteProvider` against the public Photon instance and a configurable base URL, its contract tests replay committed fixtures without a network call, it builds and lints for `wasm32`, and a release carrying it lets the Rustic Maps SDK delete its own Photon crate and its transport crate.

- [x] Crate and client (round 198): one keyless client over the Photon API (`/api`, `/reverse`), a configurable base URL, the identifying User-Agent, and the shared pipeline (`/api`), a configurable base URL, the identifying User-Agent, pacing, retry and the shared pipeline.
- [x] Contracts (round 198): forward and reverse geocoding, settled place search and type-ahead, with Photon's feature classes mapped onto `SearchQuality` and its address parts onto `Address`, settled place search and type-ahead, with the SDK's existing fixture vocabulary (feature types, extent, country-code filtering) mapped onto `SearchResult`/`SearchQuality`/`AutocompleteResult`.
- [x] Contract tests from the SDK's committed fixtures (round 198, seven wiremock cases), plus error-path and malformed-payload cases, plus error-path and malformed-payload cases.
- [x] Add the crate to the browser closure, the CI matrix, the generated READMEs and the changelog (round 198).
- [x] `openstreetmap_us_vector_source()` (round 199): the named source carries an identifying User-Agent (`OPENSTREETMAP_US_USER_AGENT`) and the service's full policy in its terms text, so a host serves OSM US through the URL-template provider without losing the request policy.
- [ ] Release the patch that carries both, and hand the Rustic Maps SDK the version to move to.

### M3 — Every provider runs in the browser · *status: open (owner-directed 2026-11-04)*

> Verify: every provider crate builds, lints and passes a browser-transport smoke under `wasm32-unknown-unknown`; the CI browser closure lists all provider crates; no provider crate depends on `tokio` or `reqwest` directly; and the README's platform matrix says so.

The point of the transport seam is that a provider is written once and reaches every target the host has. Today only `everymap-core` and the four provider crates written after the seam (MapQuest, OSRM, Valhalla, URL-template ×2) are in the browser closure; HERE, Google, TomTom, MapBox and Radar are native-only because of a dependency entry, not because of their code.

**Research (2026-11-04).** All five original providers already delegate their HTTP work to `everymap_core::client::ProviderClient` through `provider_client!`; none of their sources names `reqwest` or `tokio`. Their `[dependencies]` still list `tokio = { workspace = true }` (workspace `features = ["full"]`) and `reqwest`, which is what pulls `mio` into a wasm build and fails it. Removing those two unused entries makes all five compile for `wasm32-unknown-unknown` — verified with `cargo check` on each. Two runtime facts remain to close: one wall-clock read, and the absence of any browser test.

- [x] Remove the unused `tokio`/`reqwest` dependencies from the five original provider crates (round 194). `provider_client!` no longer names `::reqwest::ClientBuilder`; it takes `$crate::client::ClientBuilder`, which `everymap-core` re-exports natively, so a provider crate needs neither dependency.
- [x] Replace the one wall-clock read in a provider (round 194): HERE's tour request formats the departure time from `web_time::SystemTime`, the same clock the rate limiter uses, and no longer reads `chrono::Utc::now()`.
- [x] Add every provider crate to the browser closure in CI (round 194); all eleven now build and lint for `wasm32-unknown-unknown`.
- [ ] Browser runtime proof, not just compilation: a headless-browser smoke (wasm-pack test or a small harness on the shared client) that drives at least one request per provider through the fetch transport against a local server, including one request that reads the clock.
- [x] A CI grep gate that fails when a provider crate's `src` names `reqwest` or `tokio` (round 194, `scripts/check-provider-transport-isolation.sh`), so the seam cannot erode again.
- [ ] Docs: the README browser matrix, the blueprint's platform notes, the CHANGELOG entry and each crate's support line say every provider is target-agnostic, while the CLI and the bench stay native applications.
- [ ] Release the patch that carries it, and record in `docs/rfc/0001-transport-seam-and-wasm.md` that the provider set is complete on both targets.
- [x] First crates.io release (v0.2.1): publish core + 5 providers + cli via `scripts/publish-crates.sh` / `release.yml`