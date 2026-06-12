# Changelog

## [Unreleased]

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
- Radar search `bounding_box`, routing `alternatives` / `avoid`, and tour `transport_mode` now wired from core options.
- MapBox imaging `format` and tour `provider_extra` now read from core options.
- TomTom tour `TourOptions` no longer ignored — `transport_mode` and `provider_extra` extracted.
- Documentation: implementation count 35, test count 624 across all docs.

### Removed
- Obsolete `tmp/` planning documents.

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
