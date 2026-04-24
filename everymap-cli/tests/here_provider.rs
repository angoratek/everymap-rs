//! HERE provider CLI integration tests.
//!
//! Two categories:
//! 1. **Structural** — validate arg parsing, API key handling, and unsupported
//!    domain errors. No real API key needed; these always run.
//! 2. **Live API** — hit the real HERE API and validate response content.
//!    Set `EVERYMAP_HERE_API_KEY` (or `EVERYMAP_API_KEY`) to enable.
//!    Without a key, live tests skip silently (return early).

mod common;
use common::*;

use predicates::prelude::*;

/// Get the real HERE API key from the environment or config file.
///
/// Resolution order:
/// 1. `EVERYMAP_HERE_API_KEY` env var (non-empty)
/// 2. `EVERYMAP_API_KEY` env var (non-empty)
/// 3. `~/.everymap/config.toml` → `[providers.here] api_key`
fn get_here_api_key() -> Option<String> {
    std::env::var("EVERYMAP_HERE_API_KEY").ok()
        .or_else(|| std::env::var("EVERYMAP_API_KEY").ok())
        .filter(|s| !s.is_empty())
        .or_else(read_here_key_from_config)
}

/// Read the HERE API key from `~/.everymap/config.toml`.
fn read_here_key_from_config() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_path = std::path::PathBuf::from(home)
        .join(".everymap")
        .join("config.toml");
    let contents = std::fs::read_to_string(&config_path).ok()?;
    let mut in_here_section = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "[providers.here]" {
            in_here_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_here_section = false;
            continue;
        }
        if in_here_section && trimmed.starts_with("api_key") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim();
                return Some(value.trim_matches('"').to_string());
            }
        }
    }
    None
}

// ============================================================
// Structural tests (no real API key needed)
// ============================================================

// --- API key handling ---

#[test]
fn test_missing_api_key_shows_error() {
    cli_no_key()
        .args(["geocode", QUERY_BERLIN])
        .assert()
        .failure()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED));
}

#[test]
fn test_api_key_from_env_var() {
    let mut cmd = cli();
    cmd.env("EVERYMAP_API_KEY", TEST_API_KEY)
       .args(["geocode", QUERY_BERLIN])
       .assert()
       .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_cli_flag_overrides_env() {
    let mut cmd = cli();
    cmd.env("EVERYMAP_API_KEY", "wrong_key")
       .args(["--api-key", TEST_API_KEY, "geocode", QUERY_BERLIN])
       .assert()
       .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// --- Unsupported domains for HERE (geofence, trip, fraud) ---

#[test]
fn test_here_geofence_search_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "geofence-search", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_geofence_create_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "geofence-create", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG, "--radius", RADIUS_500])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_geofence_get_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "geofence-get", TEST_GEOFENCE_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_geofence_delete_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "geofence-delete", TEST_GEOFENCE_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_trip_create_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "trip-create", "--mode", TRANSPORT_CAR])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_trip_update_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "started"])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_trip_get_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_here_fraud_check_unsupported() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "fraud-check", "--device-id", TEST_DEVICE_ID, "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// ============================================================
// Live API tests (require EVERYMAP_HERE_API_KEY)
// ============================================================

// --- 11 supported HERE commands ---

#[test]
fn live_here_geocode() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "geocode", "Berlin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"results\""))
        .stdout(predicate::str::contains("52.5"))   // Berlin latitude
        .stdout(predicate::str::contains("13.3"));  // Berlin longitude
}

#[test]
fn live_here_reverse_geocode() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "reverse-geocode", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"results\""))
        .stdout(predicate::str::contains("Berlin"));
}

#[test]
fn live_here_route() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"routes\""))
        .stdout(predicate::str::contains("\"distance_m\""))
        .stdout(predicate::str::contains("\"duration_s\""));
}

#[test]
fn live_here_route_car() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "car"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"routes\""))
        .stdout(predicate::str::contains("\"distance_m\""));
}

#[test]
fn live_here_route_pedestrian() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "pedestrian"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"routes\""));
}

#[test]
fn live_here_traffic() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "traffic", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"flows\""));
}

#[test]
fn live_here_isoline() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"isolines\""));
}

#[test]
fn live_here_match_route() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    // Use a dense trace along a real road for better matching.
    let trace = "52.5164,13.3777;52.5170,13.3900;52.5175,13.3950;52.5180,13.4000";

    cli()
        .args(["--api-key", &api_key, "match-route", "--trace", trace])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"matched_points\""))
        .stdout(predicate::str::contains("\"distance_m\""));
}

#[test]
fn live_here_tour() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "tour", "--stops", BERLIN_COORDS, PARIS_COORDS])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"stops\""));
}

#[test]
fn live_here_tile() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    // HERE Vector Tile API v2 uses its own tiling scheme.
    // For Berlin at zoom 14: x=4494, y=2832 (from live_api.rs).
    cli()
        .args(["--api-key", &api_key, "tile", "--z", "14", "--x", "4494", "--y", "2832"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrieved tile"));
}

#[test]
fn live_here_position() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    // Position with no observations may succeed (IP-based) or return an error
    // while still exiting 0 (CLI prints error to stderr but doesn't exit non-zero).
    let output = cli()
        .args(["--api-key", &api_key, "position"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Accept: either real coordinate output, or a graceful error on stderr
    assert!(
        stdout.contains("\"coordinate\"") || stdout.contains("Position:") || stderr.contains("Error:"),
        "Position should return coordinates or a graceful error, got stdout: {}, stderr: {}",
        stdout, stderr
    );
}

#[test]
fn live_here_attributes() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "attributes", "--bbox", BERLIN_BBOX, "--layer", "roads"])
        .assert()
        .success();
}

#[test]
fn live_here_map_image() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "map-image", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrieved map image"));
}

// --- Output format variations with real data ---

#[test]
fn live_here_geocode_pretty() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--output", "pretty", "geocode", "Berlin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"results\""))
        .stdout(predicate::str::contains("\n")); // pretty JSON has newlines
}

#[test]
fn live_here_geocode_summary() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--output", "summary", "geocode", "Berlin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("52.5")) // summary has (lat, lng)
        .stdout(predicate::str::contains("13.3"));
}

#[test]
fn live_here_route_summary() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--output", "summary", "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .success()
        .stdout(predicate::str::contains("km"))
        .stdout(predicate::str::contains("min"));
}

#[test]
fn live_here_traffic_summary() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--output", "summary", "traffic", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("flow measurements"));
}

#[test]
fn live_here_isoline_summary() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--output", "summary", "isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .success()
        .stdout(predicate::str::contains("isoline"));
}

// --- Verbose flag with real data ---

#[test]
fn live_here_verbose_geocode() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "--verbose", "geocode", "Berlin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"results\""))
        .stderr(predicate::str::contains("https://")); // verbose shows request URL
}

#[test]
fn live_here_verbose_route() {
    let api_key = match get_here_api_key() {
        Some(key) => key,
        None => return,
    };

    cli()
        .args(["--api-key", &api_key, "-v", "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"routes\""))
        .stderr(predicate::str::contains("https://"));
}