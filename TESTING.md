# EveryMap-RS Testing Guide

## Quick Start

```bash
cargo test                    # Run 654 tests (unit + contract + CLI integration + bench; 9 more skipped)
cargo clippy -- -D warnings   # Lint (must pass clean)
cargo build                   # Build all 8 workspace crates
```

## Test Categories

### 1. Unit Tests (inline `#[cfg(test)]` modules)

Located inside source files in `everymap-core/src/domains/*.rs` and provider crates. Test:
- Option types default construction
- Serde serialization/deserialization roundtrips
- Edge cases (zero values, large coordinates, empty collections)
- All enum variants distinct and roundtrip-safe
- Provider-specific type conversions (`From<ProviderType> for CoreType`)

Run per crate:
```bash
cargo test -p everymap-core --lib
cargo test -p everymap-providers-here --lib
cargo test -p everymap-providers-google --lib
cargo test -p everymap-providers-tomtom --lib
cargo test -p everymap-providers-mapbox --lib
cargo test -p everymap-providers-radar --lib
```

### 2. Contract Tests (wiremock-based)

These use `wiremock` to mock HTTP servers and validate that provider implementations correctly deserialize real API response shapes. **No API keys required.**

Each provider crate has a `tests/` directory with per-domain contract test files:

| Provider | Test File | Domains Covered |
|----------|-----------|-----------------|
| HERE | `tests/search_contract.rs` | geocode, reverse-geocode, discover, autosuggest |
| HERE | `tests/routing_contract.rs` | route |
| HERE | `tests/traffic_contract.rs` | flow, incidents |
| HERE | `tests/positioning_contract.rs` | locate |
| HERE | `tests/isoline_contract.rs` | isoline |
| HERE | `tests/matching_contract.rs` | match-route |
| HERE | `tests/tour_contract.rs` | tour |
| HERE | `tests/tiling_contract.rs` | tile |
| HERE | `tests/attributes_contract.rs` | road attributes |
| HERE | `tests/imaging_contract.rs` | map-image |
| HERE | `tests/ext_contract.rs` | extension traits (discover, flow, incidents, locate, solve, async, health, version, road attrs) |
| HERE | `tests/error_cases.rs` | unsupported domain stubs |
| HERE | `tests/http_errors.rs` | HTTP 401/403/429/500/404 |
| Google | `tests/search_contract.rs` | geocode, reverse-geocode |
| Google | `tests/routing_contract.rs` | route |
| Google | `tests/matching_contract.rs` | match-route |
| Google | `tests/positioning_contract.rs` | geolocate |
| Google | `tests/attributes_contract.rs` | snapped speed limits |
| Google | `tests/imaging_contract.rs` | static map image |
| Google | `tests/ext_contract.rs` | extension traits |
| Google | `tests/http_errors.rs` | HTTP 401/403/429/500 |
| TomTom | `tests/search_contract.rs` | geocode, reverse-geocode (v6 format) |
| TomTom | `tests/routing_contract.rs` | route |
| TomTom | `tests/traffic_contract.rs` | flow, incidents |
| TomTom | `tests/isoline_contract.rs` | reachable range (distanceBudgetInMeters) |
| TomTom | `tests/matching_contract.rs` | snap to roads (GeoJSON projectedPoints) |
| TomTom | `tests/tour_contract.rs` | waypoint optimization (optimizedOrder) |
| TomTom | `tests/tiling_contract.rs` | map tile |
| TomTom | `tests/imaging_contract.rs` | static map image |
| TomTom | `tests/ext_contract.rs` | extension traits |
| TomTom | `tests/http_errors.rs` | HTTP 401/403/429/500 |
| MapBox | `tests/search_contract.rs` | geocode, reverse-geocode (v6 format) |
| MapBox | `tests/routing_contract.rs` | route |
| MapBox | `tests/isoline_contract.rs` | isochrone |
| MapBox | `tests/matching_contract.rs` | map matching |
| MapBox | `tests/tour_contract.rs` | optimization |
| MapBox | `tests/tiling_contract.rs` | vector tile |
| MapBox | `tests/imaging_contract.rs` | static image |
| MapBox | `tests/ext_contract.rs` | permanent geocode, batch geocode, route profile |
| MapBox | `tests/http_errors.rs` | HTTP 401/403/429/500 |
| Radar | `tests/search_contract.rs` | geocode, reverse-geocode |
| Radar | `tests/routing_contract.rs` | route (snake_case step fields) |
| Radar | `tests/matching_contract.rs` | match-route |
| Radar | `tests/tour_contract.rs` | optimization |

Run per provider:
```bash
cargo test -p everymap-providers-here   # ~25 contract tests
cargo test -p everymap-providers-google # ~14 contract tests
cargo test -p everymap-providers-tomtom  # ~14 contract tests
cargo test -p everymap-providers-mapbox  # ~14 contract tests
cargo test -p everymap-providers-radar   # ~8 contract tests
```

### 3. Error Case Tests

- **Unsupported domain stubs**: Verify that calling an unsupported domain on a provider returns `EveryMapError::UnsupportedDomain` with correct provider name and domain name.
- **HTTP error handling**: Verify that HTTP 401/403/429/500/404 responses are correctly mapped to `EveryMapError` variants (`AuthError`, `RateLimited`, `ProviderError`, etc.).

### 4. CLI Integration Tests

The CLI crate uses `assert_cmd` and `predicates` for end-to-end testing of the command-line interface.

### 5. Live API Tests

Tests that hit real API endpoints. Require valid API keys.

#### Environment Setup

Create a `.env` file in the project root (see `.env.example`):
```bash
EVERYMAP_HERE_API_KEY=your-here-key
EVERYMAP_GOOGLE_API_KEY=your-google-key
EVERYMAP_TOMTOM_API_KEY=your-tomtom-key
EVERYMAP_MAPBOX_API_KEY=your-mapbox-key
EVERYMAP_RADAR_API_KEY=your-radar-key
```

Or configure `~/.everymap/config.toml`:
```toml
[providers.here]
api_key = "your-here-key"
[providers.tomtom]
api_key = "your-tomtom-key"
[providers.mapbox]
api_key = "your-mapbox-key"
```

#### Core Integration Tests (gated behind feature flag)

```bash
cargo test -p everymap-core --features integration -- --ignored
```

#### HERE Live Tests

Located in `everymap-providers-here/tests/live_api.rs`. Silently skip if `EVERYMAP_HERE_API_KEY` is not set.

#### CLI Smoke Testing

Test each provider's supported domains via the CLI:

> **Note:** Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand.

```bash
# --- HERE (10 domains) ---
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary geocode "Berlin"
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary reverse-geocode --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary route --origin "52.52,13.40" --destination "48.85,2.35"
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary traffic --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary isoline --lat 52.52 --lng 13.40 --range 1000
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary match-route --trace "52.5164,13.3777;52.5170,13.3900;52.5175,13.3950" --transport car
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary tour --stops "52.5163,13.3777" "52.5165,13.3810" "52.5200,13.4050"
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary position
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary tile --z 14 --x 4494 --y 2832 --layer base
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary attributes --bbox "52.4,13.2;52.6,13.5" --layer roads
cargo run -p everymap-cli -- --provider here --api-key $KEY --output summary map-image --lat 52.52 --lng 13.40 --zoom 14

# --- TomTom (8 domains) ---
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary geocode "Berlin"
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary reverse-geocode --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary route --origin "52.52,13.40" --destination "48.85,2.35" --transport car
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary traffic --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary isoline --lat 52.52 --lng 13.40 --range 1000
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary match-route --trace "52.520,13.395;52.521,13.397;52.522,13.400;52.525,13.405" --transport car
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary tour --stops "52.5163,13.3777" "52.5165,13.3810" "52.5200,13.4050"
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary tile --z 14 --x 8800 --y 5374
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --output summary map-image --lat 52.52 --lng 13.40 --zoom 14

# --- MapBox (7 domains) ---
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary geocode "Berlin"
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary reverse-geocode --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary route --origin "52.52,13.40" --destination "48.85,2.35" --transport car
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary isoline --lat 52.52 --lng 13.40 --range 30
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary match-route --trace "52.520,13.395;52.521,13.397;52.522,13.400;52.525,13.405" --transport car
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary tile --z 14 --x 8800 --y 5374
cargo run -p everymap-cli -- --provider mapbox --api-key $KEY --output summary map-image --lat 52.52 --lng 13.40 --zoom 14

# --- Google (6 domains) ---
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary geocode "Berlin"
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary reverse-geocode --lat 52.52 --lng 13.40
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary route --origin "52.52,13.40" --destination "48.85,2.35" --transport car
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary match-route --trace "52.520,13.395;52.521,13.397;52.522,13.400;52.525,13.405" --transport car
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary position
cargo run -p everymap-cli -- --provider google --api-key $KEY --output summary map-image --lat 52.52 --lng 13.40 --zoom 14

# --- Radar (4 domains) ---
cargo run -p everymap-cli -- --provider radar --api-key $KEY --output summary geocode "New York"
cargo run -p everymap-cli -- --provider radar --api-key $KEY --output summary reverse-geocode --lat 40.71 --lng=-74.01
cargo run -p everymap-cli -- --provider radar --api-key $KEY --output summary route --origin "40.71,-74.01" --destination "42.36,-71.06" --transport car
cargo run -p everymap-cli -- --provider radar --api-key $KEY --output summary match-route --trace "40.71,-74.01;40.72,-74.00;40.73,-73.99" --transport car
cargo run -p everymap-cli -- --provider radar --api-key $KEY --output summary tour --stops "40.71,-74.01" "40.75,-73.99" "40.78,-73.96"
```

**Note**: Use `--lng=VALUE` (with `=`) for negative longitudes to avoid CLI argument parsing issues.

#### Verbose Mode

Add `--verbose` / `-v` to any CLI command to see:
- Request URL (redacted API key)
- Raw response body
- Response timing

```bash
cargo run -p everymap-cli -- --provider tomtom --api-key $KEY --verbose reverse-geocode --lat 52.52 --lng 13.40
```

### 6. Benchmarks

```bash
# Run all benchmarks for all configured providers
cargo run -p everymap-bench -- --all --here-key $KEY --tomtom-key $KEY

# Benchmark a specific domain
cargo run -p everymap-bench -- --domain routing --here-key $KEY

# Output formats: table (default), json, markdown
cargo run -p everymap-bench -- --all --output markdown --here-key $KEY
```

## Nextest (Enhanced Test Runner)

[cargo-nextest](https://nexte.st/) provides faster, more informative test output.

### Install

```bash
cargo install cargo-nextest --locked
```

### Run

```bash
# All tests (default profile: retries=1, slow-timeout=30s, no fail-fast)
cargo nextest run --all-features

# CI profile (fail-fast, no retries, JUnit XML report)
cargo nextest run --all-features --profile ci

# Dev-friendly wrapper script
./scripts/test.sh              # all tests
./scripts/test.sh --here       # only HERE provider
./scripts/test.sh --cli        # only CLI crate
./scripts/test.sh --ci         # CI profile with JUnit XML
./scripts/test.sh --live       # include live API tests
```

### Profiles

Defined in `.config/nextest.toml`:

| Profile | fail-fast | retries | slow-timeout | JUnit XML |
|---------|-----------|---------|--------------|-----------|
| default | false | 1 | 30s | no |
| ci      | true  | 0 | 30s | `target/nextest/junit.xml` |

## CI Pipeline

The CI runs 9 parallel jobs (`.github/workflows/ci.yml`):

1. **fmt**: `cargo fmt --all -- --check`
2. **clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **docs**: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`
4. **audit**: `cargo deny check` (advisories, licenses, bans, sources)
5. **msrv**: `cargo check --all-features` on toolchain 1.86
6. **test**: `cargo nextest run --all-features --profile ci` (JUnit XML uploaded on failure)
7. **build**: `cargo build --release -p everymap-cli`
8. **check-publish**: `./scripts/check-publish-readiness.sh --verbose`
9. **cli-validate**: `./scripts/validate-cli.sh --quiet --json` (374 CLI assertions)

## API Compatibility Notes

These are provider-specific quirks discovered during live API testing:

| Provider | Domain | Quirk |
|----------|--------|-------|
| TomTom | Reverse geocode | Response uses `addresses` array (not `results`), position is `"lat,lon"` string, boundingBox uses string coordinates |
| TomTom | Isoline | Parameter is `distanceBudgetInMeters` / `timeBudgetInSec` (not `distance` / `time`) |
| TomTom | Match-route | Points format is `lon,lat;lon,lat` (semicolon-separated, **longitude first**); requires `fields` param for projectedPoints; response is GeoJSON-based |
| TomTom | Tour | Endpoint is `/routing/waypointoptimization/1` (not `/1/api`); request uses `waypoints` (not `locations`); response returns `optimizedOrder` (array of indices) |
| MapBox | Search | v6 API puts data in `properties` object (`full_address`, `name`, `coordinates`, `bbox`, `context`), not top-level `place_name`/`text`/`center` |
| MapBox | Map-image | URL format is `/styles/v1/{user}/{style}/static/{lon},{lat},{zoom}/{w}x{h}@2x` — no `.png` extension; default style is `streets-v12` |
| MapBox | Tour | Optimization API returns "NotImplemented" on free tier |
| Radar | Routing | Step-level fields use `snake_case` (`start_location`, `bearing_after`) while leg-level uses `camelCase` (`startLocation`); API has typo `manuever` for `maneuver` |