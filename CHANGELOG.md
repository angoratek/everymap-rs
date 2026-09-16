# Changelog

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
