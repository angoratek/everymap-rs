# Changelog

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

## [0.1.0] — 2025

### Added
- Initial release with workspace architecture (8 crates).
- 10 domain traits in `everymap-core`.
- Provider implementations: HERE (10 domains), Google (6), TomTom (8), MapBox (7), Radar (4).
- 31 real implementations across 5 providers.
- CLI with 11 commands and unified `ProviderRegistry` dispatch.
- Benchmark framework covering all 10 domains.
- 575+ tests (unit + contract + CLI integration + error cases).
