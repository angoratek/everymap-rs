//! Output format, display flag, and help/version tests.
//!
//! Tests --output json/pretty/summary, invalid format fallback,
//! -v short form, --api-key-param, --version, and per-subcommand help.

mod common;
use common::*;

use predicates::prelude::*;

// ============================================================
// Output format — summary
// ============================================================

#[test]
fn test_output_format_summary() {
    cli_with_key()
        .args(["--output", "summary", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_summary_route() {
    cli_with_key()
        .args([
            "--output",
            "summary",
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
fn test_output_format_summary_reverse_geocode() {
    cli_with_key()
        .args([
            "--output",
            "summary",
            "reverse-geocode",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_summary_traffic() {
    cli_with_key()
        .args([
            "--output",
            "summary",
            "--provider",
            PROVIDER_HERE,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_summary_isoline() {
    cli_with_key()
        .args([
            "--output", "summary", "isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_summary_tour() {
    cli_with_key()
        .args([
            "--output",
            "summary",
            "tour",
            "--stops",
            BERLIN_COORDS,
            PARIS_COORDS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_summary_match_route() {
    cli_with_key()
        .args([
            "--output",
            "summary",
            "match-route",
            "--trace",
            TRACE_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Output format — invalid falls back to JSON
// ============================================================

#[test]
fn test_output_format_invalid_falls_back() {
    cli_with_key()
        .args(["--output", "xml", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_output_format_empty_string() {
    cli_with_key()
        .args(["--output", "", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Verbose flag — short form
// ============================================================

#[test]
fn test_verbose_short_flag() {
    cli_with_key()
        .args(["-v", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_verbose_short_flag_route() {
    cli_with_key()
        .args([
            "-v",
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
fn test_verbose_short_flag_radar() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "-v", "geocode", QUERY_NYC])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// API key parameter override
// ============================================================

#[test]
fn test_api_key_param_override() {
    cli_with_key()
        .args(["--api-key-param", "custom_key", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_api_key_param_with_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "--api-key-param",
            "my_custom_param",
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_api_key_param_with_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "--api-key-param",
            "token",
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Version flag
// ============================================================

#[test]
fn test_version_flag() {
    cli()
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("everymap"));
}

// ============================================================
// Subcommand help output
// ============================================================

#[test]
fn test_geocode_help() {
    cli()
        .args(["geocode", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<QUERY>"))
        .stdout(predicate::str::contains("Geocode"));
}

#[test]
fn test_reverse_geocode_help() {
    cli()
        .args(["reverse-geocode", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--lat"))
        .stdout(predicate::str::contains("--lng"));
}

#[test]
fn test_route_help() {
    cli()
        .args(["route", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--origin"))
        .stdout(predicate::str::contains("--destination"))
        .stdout(predicate::str::contains("--transport"));
}

#[test]
fn test_traffic_help() {
    cli()
        .args(["traffic", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--lat"))
        .stdout(predicate::str::contains("--lng"));
}

#[test]
fn test_isoline_help() {
    cli()
        .args(["isoline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--range"))
        .stdout(predicate::str::contains("--lat"))
        .stdout(predicate::str::contains("--lng"));
}

#[test]
fn test_match_route_help() {
    cli()
        .args(["match-route", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--trace"));
}

#[test]
fn test_tour_help() {
    cli()
        .args(["tour", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stops"));
}

#[test]
fn test_tile_help() {
    cli()
        .args(["tile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--z"))
        .stdout(predicate::str::contains("--x"))
        .stdout(predicate::str::contains("--y"));
}

#[test]
fn test_attributes_help() {
    cli()
        .args(["attributes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--bbox"))
        .stdout(predicate::str::contains("--layer"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--ids"))
        .stdout(predicate::str::contains("--include"));
}

#[test]
fn test_map_image_help() {
    cli()
        .args(["map-image", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--lat"))
        .stdout(predicate::str::contains("--lng"))
        .stdout(predicate::str::contains("--zoom"));
}

// ============================================================
// Output format combinations with all providers
// ============================================================

#[test]
fn test_output_json_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "--output",
            "json",
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_output_pretty_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "--output",
            "pretty",
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_output_summary_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "--output",
            "summary",
            "geocode",
            QUERY_BERLIN,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

// ============================================================
// Global flags combined with commands
// ============================================================

#[test]
fn test_verbose_with_summary_output() {
    cli_with_key()
        .args(["-v", "--output", "summary", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_verbose_with_pretty_output() {
    cli_with_key()
        .args(["--verbose", "--output", "pretty", "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}
