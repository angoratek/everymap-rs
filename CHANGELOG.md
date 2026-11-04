# Changelog

## [Unreleased]

## [0.3.0] — 2026-11-04

The provider layer this release exists for: a transport seam with a browser implementation, navigation-grade routing and typed tile contracts, bounding-box traffic, autocomplete/matrix/elevation/offline-routing traits, and the MapQuest, OSRM, Valhalla and URL-template provider crates the Rustic Maps SDK consumes.

### Added
- Provider clients expose the shared pipeline: `set_rate_limiter`, `set_retry_policy`, `set_request_timeout` and `set_observer` are now on the `provider_client!`-generated clients and on the MapQuest client, so a host can carry a declared budget without rebuilding the provider.
- `SearchQuality` on `SearchResult`: the nine granularity levels from a rooftop point to a country, filled by the MapQuest geocoder beside the existing confidence classification.
- `PlaceSearchProvider`: a settled place-search contract beside `Geocoder` and `AutocompleteProvider`, implemented by the MapQuest provider with the documented fifteen-result cap.
- Traffic incidents carry `start_coordinate` and `end_coordinate` when the provider reports a position; the MapQuest incident projection fills them.
- Named public vector sources: `openstreetmap_us_vector_source` and `openfreemap_vector_source` construct the OSM US Sourdough and OpenFreeMap planet descriptors with their templates, attribution, terms and encoding.
- A process-wide request observer in `everymap-core`: `install_global_request_observer` gives every provider client one redacting observer without threading it through each constructor, and a client's own observer still wins.
- `UnitSystem` on `RouteOptions`: a route can ask for metric or imperial distances while the core result stays in meters; the MapQuest router requests `unit=m` and converts the answer back.
- `TokenBucket::with_maximum_wait`: a bucket can refuse instead of waiting past a ceiling, and the shared client reports that refusal as `RateLimited`.
- `everymap-providers-mapquest`: `Geocoder`, `Router` with ordered waypoints and maneuver legs, `DistanceMatrixProvider`, `AutocompleteProvider`, `TrafficProvider` (bounding-box flow with geometry and congestion levels plus incidents) and `RasterTileProvider` implementations over the MapQuest web services, with a shared key-authenticated client, user agent, typed quality/congestion/severity mappings, committed fixtures and wiremock contract tests.
- `everymap-providers-url-template-raster`: a `RasterTileProvider` over any `{z}/{x}/{y}` raster URL template, with optional `{s}` subdomain/scheme substitution, attribution, terms, declared request headers, token-bucket pacing and typed tiles that echo the requested format and resolution.
- `everymap-providers-url-template-vector`: a `VectorTileProvider` over any `{z}/{x}/{y}` vector URL template, with optional `{s}` subdomain/scheme substitution, a `VectorTileSourceDescriptor` (cache namespace, attribution, zoom span, wire encoding, style hints), declared request headers, token-bucket pacing and gzip-encoding sniffing.
- `everymap-providers-osrm`: a `Router` implementation over the OSRM HTTP route service with ordered waypoints, per-leg steps, maneuver classification, authored instructions, GeoJSON shapes and the service's `code` error envelope.
- `everymap-providers-valhalla`: a `Router` implementation over the Valhalla `/route` endpoint with POST JSON requests, costing options, precision-6 polyline shapes and the full maneuver table.
- Traffic queries honour a bounding box: `TrafficOptions::bounding_box` asks for exactly the ground a viewport draws, and `TrafficFlow` carries an ordered `geometry` polyline and a `CongestionLevel` classification (`from_jam_factor` maps a HERE-style 0–10 score). HERE and TomTom conversions fill both.
- Autocomplete in `everymap-core`: the `AutocompleteProvider` trait with `AutocompleteOptions` (cap, language, preferred box, country codes) and `AutocompleteResult`/`AutocompleteResponse`, where a suggestion may be a region with no single point.
- Distance matrix in `everymap-core`: the `DistanceMatrixProvider` trait with `DistanceMatrixOptions` (transport mode, strategy) and row-major `DistanceMatrixEntry` results.
- Elevation in `everymap-core`: the `ElevationProvider` trait returning one ordered `ElevationSample` per requested point.
- Offline routing in `everymap-core`: the `OfflineRoutingProvider` trait (graph loading from bytes or a path, navigation-grade route results) with the engine and format left to each implementation.
- Navigation-grade routing in `everymap-core`: `Router::calculate_route_with_waypoints` for ordered waypoints (two-endpoint default that reports intermediate stops as unsupported), `Router::route_capabilities`, `RouteLeg`, `ManeuverType` with stable kebab-case labels, `TurnDirection`, `LaneGuidance`, `RouteStrategy`, and serde-defaulted `legs`, `maneuver_type`, `speech_instruction`, `lane_guidances` and `shape` fields on the route records.
- Typed raster tiles in `everymap-core`: `RasterTileProvider`, `RasterTileRequest`, `RasterTileData` (echoing format and resolution), `RasterImageFormat` and `RasterTileResolution`.
- Vector tiles in `everymap-core`: `VectorTileProvider`, `VectorTileSourceDescriptor` (cache namespace, attribution, zoom span, wire encoding, style hints), `VectorTileData` with gzip encoding sniffing, and `VectorTileEncoding`.
- Contract tests with committed fixtures and an in-process transport fake; no test opens a socket.

## [0.2.2] — 2026-09-15

### Added
- `OAuth2Provider` in `everymap-core`: OAuth 2.0 client-credentials flow with cached access tokens (30s refresh skew, 3600s fallback), `Authorization: Bearer` injection, and zeroized client secrets/tokens.
- `Moderate` variant on core `IncidentSeverity`; TomTom "moderate" traffic severity no longer downgraded to `Minor`.
- `width`/`height` fields on core `ImageOptions` (per-axis override of the size argument) and `--width`/`--height` flags on the `map-image` CLI command; wired to HERE, Google, TomTom, and MapBox static image APIs.
- `everymap_core::provider_client!` macro generating provider client structs, deduplicating ~330 lines of client boilerplate across all 5 provider crates (public API unchanged).
- Per-crate READMEs on crates.io, assembled from shared fragments in `docs/readme/` via `scripts/generate-readmes.sh` (`--check` mode for CI).

### Changed
- Google routing migrated from the legacy Directions API to Routes API v2 (`routes.googleapis.com/directions/v2:computeRoutes`, POST JSON body, required `X-Goog-FieldMask` header, API key as `key` query param); `transport_mode` now populated from `travelMode`; `provider_extra` `waypoints`→`intermediates`, `optimize_waypoints`→`optimizeWaypoints`, `units`→METRIC/IMPERIAL; `arrival_time` and avoid Tunnels/DirtRoads warn as unsupported.
- Radar routing: core `avoid` (tolls/highways/ferries) and `alternatives` fields now wired to the Directions API (previously `provider_extra`-only with warnings); unsupported avoid types (tunnels, dirt roads) log a precise warning; `provider_extra` values remain as overrides.

### Released
- Published all 7 crates to crates.io (v0.2.2): `everymap-core`, the 5 provider crates, and `everymap-cli`, plus prebuilt CLI binaries for 5 targets on the [GitHub Release](https://github.com/angoratek/everymap-rs/releases/tag/v0.2.2). First release with per-crate READMEs on crates.io.

## [0.2.1] — 2026-09-13

### Added
- `scripts/validate-cli.sh`: 374-assertion CLI validation framework with timing and JSON output.
- `SUPPORT.md`, `CODEOWNERS`, `.github/FUNDING.yml` community files.
- Module-level documentation on `everymap-core` crate (crates.io/docs.rs landing page).
- `waypoint_distance` field on `HereMatchingOptions` (replaces abbreviated `wp_dist`).
- 20+ new CLI flags: `--language`, `--limit`, `--country`, `--bbox`, `--radius`, `--alternatives`, `--avoid`, `--departure-time`, `--arrival-time`, `--heading`, `--range-type`, `--format` across all 11 commands.
- 24 `log::warn!` diagnostics for silent parameter drops across all 5 providers.
- `.github/dependabot.yml` for weekly Cargo + GitHub Actions dependency updates.

### Changed
- `wp_dist` field on `HereMatchingOptions` is now deprecated — use `waypoint_distance` instead.
- `here_opts` variable renamed to `here_options` (256 occurrences, 9 files).
- Replaced `eprintln!` with `log::debug!` / `log::warn!` in library crates (core client, MapBox provider).
- TomTom routing and isoline now wire core `AvoidType` to TomTom API params (`avoidTollRoads`, `avoidFerries`, `avoidTunnels`, `avoidMotorways`, `avoidUnpavedRoads`).
- HERE reverse geocode, imaging, and traffic now read core `radius` / `format` / `language` fields.
- Radar search `bounding_box` and tour `transport_mode` now wired from core options (routing `alternatives` / `avoid` remain available via `provider_extra` only, with warnings).
- MapBox imaging `format` and tour `provider_extra` now read from core options.
- TomTom tour `TourOptions` no longer ignored — `transport_mode` and `provider_extra` extracted.
- Documentation: implementation count 35, test count 623 across all docs.

### Added (CI/CD and release infrastructure)
- CI expanded to 9 jobs: fmt, clippy, docs (`RUSTDOCFLAGS="-D warnings"`), audit, msrv, test, build, check-publish, cli-validate.
- `deny.toml` + `cargo deny check` audit job pinned to cargo-deny 0.20.2 (advisories, licenses, bans, sources).
- MSRV bumped from 1.75 to 1.86 (edition-2024 dependencies; icu4x 2.2 requires 1.86).
- reqwest upgraded from 0.12 to 0.13 (`query` feature now opt-in; default TLS backend is rustls/aws-lc-rs).
- Security fixes: h2 0.4.19 (RUSTSEC-2026-0258), rustls-webpki 0.103.15 (RUSTSEC-2026-0098/0099/0104).
- `.github/workflows/release.yml`: tag-triggered release pipeline (check → 5-target binaries → crates.io publish → GitHub release).
- crates.io metadata: `readme`, `documentation`, and `authors` on all publishable crates; `cargo publish --locked`; index-polling between publishes replaces the fixed sleep.
- GitHub Actions bumped: checkout v7, upload-artifact v7, download-artifact v8.

### Removed
- Obsolete `tmp/` planning documents.

### Released
- Published all 7 crates to crates.io (v0.2.1): `everymap-core`, the 5 provider crates, and `everymap-cli`, plus prebuilt CLI binaries for 5 targets on the [GitHub Release](https://github.com/angoratek/everymap-rs/releases/tag/v0.2.1).

## [0.2.0] — 2026-05-03

### Added
- `DepartureTime` enum in `everymap-core` (Now, Timestamp, Iso8601) replacing fragile `Option<String>` for departure/arrival times.
- Typed transport mode mapping for HERE matching (Bus→Bus, Scooter→Motorcycle, Taxi→Taxi).
- MapBox reverse geocode radius warning (unsupported by v6).
- `SECURITY.md` and `CHANGELOG.md`.

### Fixed
- Google routing: removed invalid `indoor` mapping for DirtRoads, removed unsupported Tunnels avoid type.
- HERE traffic: `include_incidents` now actually fetches incidents (was always empty).
- CLI Attributes: `--layer` flag key fixed (was `"layer"`, now `"layers"` matching HERE provider).
- All variable name abbreviations eliminated (`opts`→`options`, `res`→`response`, `coord`→`coordinate`, etc.).

### Changed
- `RouteOptions.departure_time` and `.arrival_time` changed from `Option<String>` to `Option<DepartureTime>`.
- `MatchingOptions.departure_time` and `IsolineOptions.departure_time` changed to `Option<DepartureTime>`.

## [0.1.0] — 2026-04

### Added
- Initial release with workspace architecture (8 crates).
- 10 domain traits in `everymap-core`.
- Provider implementations: HERE (10 domains), Google (6), TomTom (8), MapBox (7), Radar (4).
- 35 real implementations across 5 providers.
- CLI with 11 commands and unified `ProviderRegistry` dispatch.
- Benchmark framework covering all 10 domains.
- 624 tests (unit + contract + CLI integration + error cases + bench).
