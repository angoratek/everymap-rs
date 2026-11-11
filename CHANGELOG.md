# Changelog

## [Unreleased]

### Fixed
- Google's Routes API requests asked for `routes.travelMode`, which the v2 API does not define; every call failed with `400 INVALID_ARGUMENT` before routing. The field mask now requests only defined fields, and a route's travel mode is taken from the request.
- Radar sent `pedestrian`, `scooter` or an enum name as the travel mode on tour planning and other values on matching, because three endpoints carried three tables. One shared mapping now serves routing, matching and tour planning, mapping every Radar-served mode and warning on the rest.
- `HttpRequest`'s derived `Debug` printed authorization headers, credential-bearing query parameters and request bodies. Its `Debug` now redacts those values and the whole body, so a request handed to a logger cannot print a key, token, signature or session.
- `MapQuestProviderConfiguration`'s derived `Debug` printed the consumer key; the key is redacted now.
- The observer's redaction vocabulary gains `session`, `sessionId`, `signature` and `sig` in both the query and response-field lists.
- The CLI's provider registry carried an unused MapQuest traffic stub beside a comment claiming MapQuest serves no traffic, and an `unreachable!` for an unknown provider key. The stub is gone and an unknown key answers a typed validation error.

### Added
- CLI commands exit non-zero when the provider call fails, so a script can tell a refused request from a successful one.
- `MapBoxTileProvider::with_tileset_id` selects the tileset a provider serves; the descriptor and the requested path follow it, and the default stays Streets v8.

### Changed
- `OAuth2Provider` holds its token lock only to read or replace the cached token, never across the token request, and each request carries a configurable timeout (ten seconds by default). A slow token endpoint can no longer block every capability that shares the credential.
- Native transport failures answer a classified `TransportError` sentence instead of `reqwest`'s own message, which printed the request URL and therefore a query-string credential. Response-body read failures take the same path.

### Fixed (continued)
- TomTom honours a caller's traffic `bounding_box` and sends the incident bbox in the documented `minLon,minLat,maxLon,maxLat` order; a radius still works when no box is supplied.
- Mapbox `permanent_geocode` sends `permanent=true`, and `batch_geocode` posts the v6 batch endpoint with every query in one request instead of issuing sequential forward lookups.
- `scripts/check-publish-readiness.sh` fails on a crate or inter-crate dependency version that differs from the workspace version.
- The release workflow runs the full CI set: formatting, clippy, the whole nextest suite, the open-source hygiene gate, the CLI validation suite and publish readiness.
- The open-source hygiene gate also scans binary files and commit metadata.

## [0.4.3] — 2026-11-10

### Changed (breaking)
- The URL-template raster provider validates the requested image format and resolution against its source declaration and refuses a mismatch with a typed validation error instead of labelling the bytes with what the caller asked for. The URL template, the tile-index validation and the pacing declaration now live once in `everymap_core::tile_source`, and both URL-template providers use that one implementation.
- `PhotonClient` gains the pacing, retry and request-timeout setters every other client has; Photon now emits the reverse-geocode language as `lang`, honours the reverse result limit, and refuses the options it cannot honour (`country_codes`, a reverse-geocode radius) with a typed validation error instead of dropping them.
- Google, Radar, TomTom and MapBox report routing capabilities, error categories, endpoints and search quality honestly: each `route_capabilities()` names the modes and the shortest strategy it actually honours; Google's HTTP-200 `REQUEST_DENIED`/`OVER_QUERY_LIMIT` bodies map to auth and rate-limit failures; Radar status codes do the same; malformed bodies map to the parse category; each provider derives every endpoint from one configurable service root; a required response field that is missing is a typed invalid-response error instead of a substituted `(0,0)`, empty leg or zero total; routes carry legs; `SearchResult.quality` is filled from each vendor's granularity; and `raw` is omitted rather than fabricated when it cannot be serialized.

### Fixed
- HERE traffic reads the contract's radius in metres and converts it to degrees, honours a caller-supplied `bounding_box` on both the flow and incident endpoints, propagates an incidents failure instead of returning an empty list, and refuses a query that names neither ground instead of substituting a silent default.
- HERE routing maps every response section into a leg with its distance, duration, decoded shape and actions-derived steps, requests turn-by-turn actions by default, returns `INVALID_POLYLINE` for a malformed shape instead of a plausible empty route, and reports the seven transport modes and the shortest strategy it honours from `route_capabilities()`.
- HERE matching and isoline responses require their documented fields instead of applying `serde(default)`; a missing response, position coordinate, isoline or range is a typed `DESERIALIZATION_ERROR`.

## [0.4.2] — 2026-11-06

### Added
- Observers can receive a bounded response excerpt. `set_response_excerpt_capture(true)` makes each client hand `ResponseObservation::response_excerpt` the first 512 bytes of the response body, truncated on a character boundary, with every credential-bearing JSON member or `name=value` assignment replaced by `***`; the switch is off by default and the excerpt is `None` while it is. Request bodies and header values stay out of the seam.
- The keyless clients report their attempts like every credentialed one: `OsrmRouter::with_observer` and `ValhallaRouter::with_observer` build a router whose client publishes each request, and `PhotonClient::set_observer` installs the same observer on the keyless Photon client. A host that shows request diagnostics can now list self-hosted and public keyless providers instead of only credentialed ones.

### Fixed
- The CLI's MapQuest arm did not install its raster-tile provider, so `tile --provider mapquest --variant raster` was refused although the crate serves raster tiles.

## [0.4.1] — 2026-11-06

### Fixed
- Mapbox, Radar and Google routes always reported an empty shape, or silently dropped points: the first two decoded the provider's encoded polyline with HERE's flexible-polyline codec and swallowed the failure, and Google's hand-rolled decoder shifted past the integer width on malformed input. `everymap-core` now carries one hardened encoded-polyline decoder (precision 5 and 6, bounded accumulator, alphabet and range checks), and the three providers return `ProviderError("INVALID_POLYLINE")` instead of a plausible empty route. Their contract fixtures carry valid shapes and assert the decoded points.
- A browser request carried the provider's `User-Agent` header, which the Fetch standard forbids: a browser that treats it as a custom header preflights the request, and a tiles endpoint without an `OPTIONS` route answers 404. `everymap-core`'s fetch transport drops the Fetch-forbidden header names before building the request.
- Radar steps aliased `manuever` onto `maneuver`, so a live payload carrying both spellings matched one field twice and the whole directions response failed to deserialize. The alias is gone.
- The recorded MapQuest directions payload carried the live session id from the capture, embedded six times (the `sessionId` field and the image URLs beside it). The payload now keeps a synthetic placeholder, and the hygiene gate refuses a session or signature token in any tracked file. The repository history and tags v0.3.0–v0.4.0 were rewritten to remove it; the 0.3.0–0.4.0 crate tarballs on crates.io predate the rewrite and are recorded in `SECURITY.md`.
- The CLI validation suite now asserts the typed tile flags the command takes (`--variant`, `--format`, `--resolution`); it still asserted the removed `--layer` flag, so the tagged 0.4.0 run's `cli-validate` job failed after the release.

## [0.4.0] — 2026-11-05

### Changed (breaking)
- One contract surface. The untyped `TileProvider`, its `TileOptions`/`TileResponse` records and `unsupported_tile!` are removed; `RasterTileProvider` and `VectorTileProvider` are the only tile contracts. HERE and MapBox serve `VectorTileProvider`: HERE requests the Vector Tile API's only output segment, `omv`, whose bytes are MVT protobuf over zoom 0–17, and MapBox requests Streets v8 MVT. TomTom serves `RasterTileProvider`, PNG at standard resolution; a JPEG or Retina request is refused. The CLI's `tile` command takes `--variant raster|vector`, defaulting to the first contract the provider serves, with the typed request's format and resolution, and the benchmark measures the same two contracts.
- The provider matrix, the crate READMEs and the CLI's descriptions use one vocabulary — Geocoding, Place search, Type-ahead, Routing, Distance matrix, Traffic, Raster tiles, Vector tiles, Elevation, Isoline, Matching, Tour, Positioning, Attributes, Imaging, Offline routing — with no distinction between providers written before or after the typed contracts. `everymap-core`'s crates.io description names the provider family.
- Every crate README opens with the same paragraph naming the eleven providers and the sixteen contracts, then carries the same three-column contract table and a sentence naming the contracts it does not serve. The five original providers state that routing takes an origin and a destination and refuses a longer request rather than dropping stops.

### Added
- `AutocompleteResult::quality`: a candidate carries the match granularity its provider classifies, on the same `SearchQuality` scale a settled search reports. Photon fills it from the feature class; MapQuest leaves it unset, and a host falls back to its own default. The field is additive and serde-defaulted, so an existing candidate literal keeps compiling with one added line.
- `docs/design/UnifiedContracts.md`: the audit behind the matrix and the rule that a new capability is a new contract for every provider.
- `scripts/check-open-source-hygiene.sh`, run in CI: the tracked tree (planning documents included) may carry no credential, personal filesystem path or private marker.

### Fixed
- The browser-closure clippy gate actually lints the closure. Every `cargo` flag in the CI step followed `--`, which hands the rest of the line to rustc: `-p` was ignored, cargo fell back to its default target set for the whole workspace, and the native test harness's full-featured `tokio` pulled `mio` into a `wasm32` build, where it refuses to compile. The step now passes its `-p` selections to cargo and only `-D warnings` after the separator.
- The dependency audit is clean again. Cargo.lock carries patched `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-root-certs` 1.0.11 and `tokio` 1.54.1, which close every advisory `cargo deny` reported: TLS session secrets in `Debug` output, trust in distrusted certificate intermediates and roots, and the `AsyncFd` I/O-safety issue.

## [0.3.2] — 2026-11-05

### Fixed
- The URL-template raster provider applies `TileSourcePacing::maximum_wait_seconds`. 0.3.1 added the field and honoured it in the vector provider; the raster provider kept its own bucket construction and ignored it, so a paced raster source still waited for its refill instead of refusing at the declared ceiling.

### Added
- `openstreetmap_us_vector_source()` now carries an identifying `User-Agent` (`OPENSTREETMAP_US_USER_AGENT`) and the tileservice's full usage policy in its terms text, so a host can serve the source through the URL-template provider without losing the request policy the dedicated crate carried.
- `everymap-providers-photon`: a keyless Photon provider covering forward and reverse geocoding, settled place search and type-ahead over the public instance or a self-hosted deployment, with Photon's feature classes mapped onto the core's quality scale and its GeoJSON `FeatureCollection` projected onto the core records. It builds for the browser and joins the CI closure; seven wiremock contract cases replay the committed fixtures.
- `everymap-cli` and `everymap-bench` select MapQuest, OSRM and Valhalla beside the five original providers. MapQuest serves geocoding, routing and traffic; OSRM and Valhalla serve routing and take their deployment base URL in the credential slot (an empty value reaches the documented local default). A domain a provider does not serve answers with the documented unsupported-domain error. The CLI's provider validation moved ahead of credential resolution so the keyless engines need no key, and the error keeps its documented "API key required" prefix.
- `unsupported_geocoder!` and `unsupported_matcher!` join the unsupported-domain macros, so a routing-only provider can declare the two contracts it lacks.

### Changed
- Every provider crate builds for the browser. The five original providers already delegated to the shared `ProviderClient`; their unused `tokio` and `reqwest` dependencies were what pulled a native-only runtime into a `wasm32` build, and `provider_client!` now names `everymap-core`'s re-exported `ClientBuilder` instead of `reqwest` directly. HERE's tour request reads the wall clock through `web-time` rather than `chrono::Utc::now()`, which is unimplemented in a browser. The CI browser closure now covers core and all eleven providers, and `scripts/check-provider-transport-isolation.sh` fails a provider crate that names `reqwest` or `tokio` directly.

## [0.3.1] — 2026-11-04

### Fixed
- The browser target reads time through `web-time` instead of `std::time`. `Instant::now()` is unimplemented on `wasm32-unknown-unknown` and panicked, so every provider call — a paced tile source, a MapQuest request, an OSRM or Valhalla route — trapped the module even though the crate built for the target. `everymap-core`'s rate limiter, request pipeline and OAuth2 token clock now use the same monotonic clock the SDK's own transport uses; native builds are unchanged.

### Added
- `TileSourcePacing::maximum_wait_seconds`: a tile source can bound how long a fetch waits for its token. Past the ceiling the shared client reports `RateLimited` instead of stalling, which restores a host's ability to refuse an over-budget tile request in bounded time. The field is optional and `TileSourcePacing::new` leaves it unset, so a declaration that does not use it is unchanged.

## [0.3.0] — 2026-11-04

The provider layer this release exists for: a transport seam with a browser implementation, navigation-grade routing and typed tile contracts, bounding-box traffic, autocomplete/matrix/elevation/offline-routing traits, and the MapQuest, OSRM, Valhalla and URL-template provider crates a host application consumes.

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
