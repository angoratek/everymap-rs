//! Complete provider×domain matrix tests.
//!
//! Tests that every unsupported domain returns an appropriate error for every
//! provider that doesn't support it.

mod common;
use common::*;

use predicates::prelude::*;

// ============================================================
// Traffic domain — unsupported for: google, mapbox, radar
// ============================================================

#[test]
fn test_traffic_unsupported_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_GOOGLE,
            "traffic",
        )));
}

#[test]
fn test_traffic_unsupported_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_MAPBOX,
            "traffic",
        )));
}

#[test]
fn test_traffic_unsupported_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "traffic",
        )));
}

// ============================================================
// Position domain — unsupported for: tomtom, mapbox, radar
// ============================================================

#[test]
fn test_position_unsupported_tomtom() {
    cli_with_key()
        .args(["--provider", PROVIDER_TOMTOM, "position"])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_TOMTOM,
            "positioning",
        )));
}

#[test]
fn test_position_unsupported_mapbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_MAPBOX, "position"])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_MAPBOX,
            "positioning",
        )));
}

#[test]
fn test_position_unsupported_radar() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "position"])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "positioning",
        )));
}

// ============================================================
// Isoline domain — unsupported for: google, radar
// ============================================================

#[test]
fn test_isoline_unsupported_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "isoline",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_GOOGLE,
            "isoline",
        )));
}

#[test]
fn test_isoline_unsupported_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "isoline",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "isoline",
        )));
}

// ============================================================
// Tour domain — unsupported for: google
// ============================================================

#[test]
fn test_tour_unsupported_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "tour",
            "--stops",
            BERLIN_COORDS,
            PARIS_COORDS,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_GOOGLE,
            "tour",
        )));
}

// ============================================================
// Tile domain — unsupported for: google, radar
// ============================================================

#[test]
fn test_tile_unsupported_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "tile",
            "--z",
            TILE_Z,
            "--x",
            TILE_X,
            "--y",
            TILE_Y,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_GOOGLE,
            "tiling",
        )));
}

#[test]
fn test_tile_unsupported_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "tile",
            "--z",
            TILE_Z,
            "--x",
            TILE_X,
            "--y",
            TILE_Y,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "tiling",
        )));
}

// ============================================================
// Attributes domain — unsupported for: tomtom, mapbox, radar
// ============================================================

#[test]
fn test_attributes_unsupported_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "attributes",
            "--bbox",
            BERLIN_BBOX,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_TOMTOM,
            "attributes",
        )));
}

#[test]
fn test_attributes_unsupported_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "attributes",
            "--bbox",
            BERLIN_BBOX,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_MAPBOX,
            "attributes",
        )));
}

#[test]
fn test_attributes_unsupported_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "attributes",
            "--bbox",
            BERLIN_BBOX,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "attributes",
        )));
}

// ============================================================
// Map-image domain — unsupported for: radar
// ============================================================

#[test]
fn test_map_image_unsupported_radar() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains(unsupported_domain_msg(
            PROVIDER_RADAR,
            "imaging",
        )));
}

// ============================================================
// Supported domain validation — confirm supported providers do NOT
// produce "does not support" errors for their supported domains
// ============================================================

#[test]
fn test_here_supports_traffic() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support traffic").not());
}

#[test]
fn test_here_supports_isoline() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
            "isoline",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support isoline").not());
}

#[test]
fn test_here_supports_position() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "position"])
        .assert()
        .stderr(predicate::str::contains("does not support position").not());
}

#[test]
fn test_tomtom_supports_traffic() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "traffic",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support traffic").not());
}

#[test]
fn test_tomtom_supports_isoline() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "isoline",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support isoline").not());
}

#[test]
fn test_tomtom_supports_tour() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "tour",
            "--stops",
            BERLIN_COORDS,
            PARIS_COORDS,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support tour").not());
}

#[test]
fn test_tomtom_supports_tile() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "tile",
            "--z",
            TILE_Z,
            "--x",
            TILE_X,
            "--y",
            TILE_Y,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support tiling").not());
}

#[test]
fn test_tomtom_supports_map_image() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support imaging").not());
}

#[test]
fn test_mapbox_supports_isoline() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "isoline",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support isoline").not());
}

#[test]
fn test_mapbox_supports_tour() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "tour",
            "--stops",
            BERLIN_COORDS,
            PARIS_COORDS,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support tour").not());
}

#[test]
fn test_mapbox_supports_tile() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "tile",
            "--z",
            TILE_Z,
            "--x",
            TILE_X,
            "--y",
            TILE_Y,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support tiling").not());
}

#[test]
fn test_mapbox_supports_map_image() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support imaging").not());
}

#[test]
fn test_google_supports_position() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "position"])
        .assert()
        .stderr(predicate::str::contains("does not support position").not());
}

#[test]
fn test_google_supports_attributes() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "attributes",
            "--bbox",
            BERLIN_BBOX,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support attributes").not());
}

#[test]
fn test_google_supports_map_image() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "map-image",
            "--lat",
            BERLIN_LAT,
            "--lng",
            BERLIN_LNG,
        ])
        .assert()
        .stderr(predicate::str::contains("does not support imaging").not());
}
