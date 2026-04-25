//! Shared constants and helpers for CLI integration tests.
//!
//! Centralizes all hardcoded values so they can be changed in one place
//! and easily varied for broader coverage. Each constant is named by
//! its semantic purpose, not its numeric value.
//!
//! Note: `#[allow(dead_code)]` is needed because each test binary only
//! imports a subset of these constants.

#![allow(dead_code)]

use assert_cmd::Command;

// ============================================================
// API keys
// ============================================================

/// Dummy API key used for argument-parsing tests (not a real key).
pub const TEST_API_KEY: &str = "test";

// ============================================================
// Provider names
// ============================================================

pub const PROVIDER_HERE: &str = "here";
pub const PROVIDER_GOOGLE: &str = "google";
pub const PROVIDER_TOMTOM: &str = "tomtom";
pub const PROVIDER_MAPBOX: &str = "mapbox";
pub const PROVIDER_RADAR: &str = "radar";

/// All supported provider names — useful for iteration.
pub const ALL_PROVIDERS: &[&str] = &[
    PROVIDER_HERE,
    PROVIDER_GOOGLE,
    PROVIDER_TOMTOM,
    PROVIDER_MAPBOX,
    PROVIDER_RADAR,
];

// ============================================================
// Geographic coordinates
// ============================================================

/// Berlin (Brandenburg Gate area) — primary test coordinate.
pub const BERLIN_LAT: &str = "52.5";
pub const BERLIN_LNG: &str = "13.3";
/// Combined "lat,lng" for route/trip/tour arguments.
pub const BERLIN_COORDS: &str = "52.5,13.3";

/// Second point near Berlin — used in traces, multi-stop tours.
pub const BERLIN2_COORDS: &str = "52.6,13.4";

/// Paris — used as route destination.
pub const PARIS_COORDS: &str = "48.8,2.3";

/// New York City — primary Radar test coordinate (correct negative longitude).
pub const NYC_LAT: &str = "40.71";
pub const NYC_LNG: &str = "-74.01";
pub const NYC_COORDS: &str = "40.71,-74.01";

/// NYC with positive lng — for `--lng` flag tests that don't use `=` syntax.
/// Geographically wrong but avoids CLI negative-arg parsing issues for simple tests.
pub const NYC_LAT_POSLNG: &str = "40.71";
pub const NYC_LNG_POS: &str = "74.0";

/// Boston — used as Radar trip destination.
pub const BOSTON_COORDS: &str = "42.36,-71.06";

// ============================================================
// Bounding boxes
// ============================================================

/// Berlin area bounding box (south,west;north,east).
pub const BERLIN_BBOX: &str = "52.4,13.2;52.6,13.5";

// ============================================================
// Tile coordinates (HERE Vector Tile API v2 tiling scheme)
// ============================================================

/// Berlin area at zoom 14 in HERE's own tiling scheme.
pub const HERE_TILE_Z: &str = "14";
pub const HERE_TILE_X: &str = "4494";
pub const HERE_TILE_Y: &str = "2832";

/// Standard web Mercator tile coords (Slippy map, used by TomTom/MapBox).
pub const TILE_Z: &str = "14";
pub const TILE_X: &str = "8800";
pub const TILE_Y: &str = "5374";

// ============================================================
// IDs
// ============================================================

pub const TEST_TRIP_ID: &str = "trip_123";
pub const TEST_GEOFENCE_ID: &str = "gf_123";
pub const TEST_DEVICE_ID: &str = "dev_1";
pub const TEST_USER_ID: &str = "user_1";

// ============================================================
// Query strings
// ============================================================

pub const QUERY_BERLIN: &str = "Berlin";
pub const QUERY_NYC: &str = "New York";

// ============================================================
// Trace strings
// ============================================================

/// Standard 2-point trace near Berlin.
pub const TRACE_BERLIN: &str = "52.5,13.3;52.6,13.4";

/// 2-point trace near NYC (correct negative longitudes).
pub const TRACE_NYC: &str = "40.71,-74.01;40.72,-74.00";

// ============================================================
// Transport modes
// ============================================================

pub const TRANSPORT_CAR: &str = "car";

// ============================================================
// Radii and ranges
// ============================================================

pub const RADIUS_500: &str = "500";
pub const RADIUS_1000: &str = "1000";
pub const ISOLINE_RANGE_DEFAULT: &str = "1000";

// ============================================================
// Geofence tags
// ============================================================

pub const TAG_STORE: &str = "store";

// ============================================================
// CLI helpers
// ============================================================

/// Create a CLI `Command` for the everymap binary.
pub fn cli() -> Command {
    Command::cargo_bin("everymap").unwrap()
}

/// Build a base CLI command with `--api-key` set to the test dummy key.
///
/// Use this for most tests that need to get past the "API key required" check.
pub fn cli_with_key() -> Command {
    let mut cmd = cli();
    cmd.arg("--api-key").arg(TEST_API_KEY);
    cmd
}

/// Build a CLI command with `EVERYMAP_API_KEY` env var removed.
///
/// Use this for tests that verify the "API key required" error path.
/// Removes the env var that clap reads for `--api-key` so the CLI
/// actually sees a missing key.
pub fn cli_no_key_env() -> Command {
    let mut cmd = cli();
    cmd.env_remove("EVERYMAP_API_KEY");
    cmd
}

/// Build a CLI command with no API key available from any source.
///
/// Removes `EVERYMAP_API_KEY` env var and sets `HOME` to a temp directory
/// so `~/.everymap/config.toml` won't be found. This forces the
/// "API key required" error path.
pub fn cli_no_key() -> Command {
    let mut cmd = cli();
    cmd.env_remove("EVERYMAP_API_KEY")
        .env("HOME", "/tmp/everymap-test-no-config");
    cmd
}

// ============================================================
// Predicate / assertion string constants
// ============================================================

/// Stderr substring when an optional domain (geofence/trip/fraud) is not available for a provider.
pub const ERR_NOT_SUPPORTED: &str = "not supported by this provider";

/// Stderr substring prefix for unsupported domain stubs.
/// Use with format!: `format!("{} does not support {}", provider, domain)`
pub const ERR_DOES_NOT_SUPPORT_PREFIX: &str = "does not support";

/// Stderr substring when provider name is invalid.
pub const ERR_UNSUPPORTED_PROVIDER: &str = "Unsupported provider";

/// Stderr substring when API key is missing.
pub const ERR_API_KEY_REQUIRED: &str = "API key required";

/// Build the "provider does not support domain" error string.
pub fn unsupported_domain_msg(provider: &str, domain: &str) -> String {
    format!("{} does not support {}", provider, domain)
}
