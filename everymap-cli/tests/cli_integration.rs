mod common;
use common::*;

use predicates::prelude::*;

// ============================================================
// Provider validation tests
// ============================================================

#[test]
fn test_unsupported_provider() {
    cli()
        .args([
            "--provider",
            "invalid",
            "--api-key",
            TEST_API_KEY,
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported provider 'invalid'"));
}

#[test]
fn test_supported_providers() {
    for provider in ALL_PROVIDERS {
        cli()
            .args([
                "--provider",
                provider,
                "--api-key",
                TEST_API_KEY,
                "geocode",
                QUERY_BERLIN,
            ])
            .assert()
            .stderr(predicate::str::contains("Unsupported provider").not());
    }
}

#[test]
fn test_default_provider_is_here() {
    cli()
        .args(["--api-key", TEST_API_KEY, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains("Unsupported provider").not());
}

#[test]
fn test_missing_api_key_no_env() {
    cli()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--api-key"));
}

// ============================================================
// Geocode command tests
// ============================================================

#[test]
fn test_geocode_requires_query() {
    cli_with_key()
        .arg("geocode")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_geocode_with_query() {
    cli_with_key()
        .arg("geocode")
        .arg(QUERY_BERLIN)
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Reverse geocode command tests
// ============================================================

#[test]
fn test_reverse_geocode_requires_lat() {
    cli_with_key()
        .args(["reverse-geocode", "--lng", BERLIN_LNG])
        .assert()
        .failure();
}

#[test]
fn test_reverse_geocode_requires_lng() {
    cli_with_key()
        .args(["reverse-geocode", "--lat", BERLIN_LAT])
        .assert()
        .failure();
}

#[test]
fn test_reverse_geocode_with_coords() {
    cli_with_key()
        .args(["reverse-geocode", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Route command tests
// ============================================================

#[test]
fn test_route_requires_origin() {
    cli_with_key()
        .args(["route", "--destination", BERLIN_COORDS])
        .assert()
        .failure();
}

#[test]
fn test_route_requires_destination() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS])
        .assert()
        .failure();
}

#[test]
fn test_route_with_valid_args() {
    cli_with_key()
        .args([
            "route",
            "--origin",
            BERLIN_COORDS,
            "--destination",
            PARIS_COORDS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_route_with_transport_mode() {
    cli_with_key()
        .args([
            "route",
            "--origin",
            BERLIN_COORDS,
            "--destination",
            PARIS_COORDS,
            "--transport",
            "bicycle",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Traffic command tests
// ============================================================

#[test]
fn test_traffic_requires_lat() {
    cli_with_key()
        .args(["traffic", "--lng", BERLIN_LNG])
        .assert()
        .failure();
}

#[test]
fn test_traffic_with_coords() {
    cli_with_key()
        .args(["traffic", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Isoline command tests
// ============================================================

#[test]
fn test_isoline_requires_lat() {
    cli_with_key()
        .args(["isoline", "--lng", BERLIN_LNG])
        .assert()
        .failure();
}

#[test]
fn test_isoline_with_default_range() {
    cli_with_key()
        .args(["isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Match route command tests
// ============================================================

#[test]
fn test_match_route_requires_trace() {
    cli_with_key().arg("match-route").assert().failure();
}

#[test]
fn test_match_route_with_trace() {
    cli_with_key()
        .args(["match-route", "--trace", TRACE_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Tour command tests
// ============================================================

#[test]
fn test_tour_requires_stops() {
    cli_with_key().arg("tour").assert().failure();
}

#[test]
fn test_tour_with_stops() {
    cli_with_key()
        .args(["tour", "--stops", BERLIN_COORDS, PARIS_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Tile command tests
// ============================================================

#[test]
fn test_tile_requires_args() {
    cli_with_key().arg("tile").assert().failure();
}

#[test]
fn test_tile_with_xyz() {
    cli_with_key()
        .args(["tile", "--z", TILE_Z, "--x", TILE_X, "--y", TILE_Y])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Map image command tests
// ============================================================

#[test]
fn test_map_image_requires_lat() {
    cli_with_key()
        .args(["map-image", "--lng", BERLIN_LNG])
        .assert()
        .failure();
}

#[test]
fn test_map_image_with_coords() {
    cli_with_key()
        .args([
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
            "--zoom",
            "14",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_map_image_with_width_height() {
    cli_with_key()
        .args([
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
            "--width",
            "1024",
            "--height",
            "768",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Attributes command tests
// ============================================================

#[test]
fn test_attributes_with_bbox() {
    cli_with_key()
        .args(["attributes", "--bbox", BERLIN_BBOX])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Output format tests
// ============================================================

#[test]
fn test_output_format_json() {
    cli_with_key()
        .args(["--output", "json", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_pretty() {
    cli_with_key()
        .args(["--output", "pretty", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Verbose flag tests
// ============================================================

#[test]
fn test_verbose_flag() {
    cli_with_key()
        .args(["--verbose", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Help output tests
// ============================================================

#[test]
fn test_help_shows_provider_options() {
    cli()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("provider"));
}

#[test]
fn test_help_shows_all_commands() {
    let output = cli().args(["--help"]).assert().success();

    let stdout = std::str::from_utf8(&output.get_output().stdout).unwrap();
    assert!(stdout.contains("geocode"));
    assert!(stdout.contains("reverse-geocode"));
    assert!(stdout.contains("route"));
    assert!(stdout.contains("traffic"));
    assert!(stdout.contains("position"));
    assert!(stdout.contains("isoline"));
    assert!(stdout.contains("match-route"));
    assert!(stdout.contains("tour"));
    assert!(stdout.contains("tile"));
    assert!(stdout.contains("attributes"));
    assert!(stdout.contains("map-image"));
}
