//! Complete provider×domain matrix tests.
//!
//! Tests that every unsupported domain returns an appropriate error for every
//! provider that doesn't support it. This covers both:
//! - Universal domains with stubs: "Error: {provider} does not support {domain}"
//! - Optional domains (None): "Error: ... is not supported by this provider. Use --provider radar"

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
// Geofence domain (optional, None for non-radar) — 4 commands × 4 providers
// ============================================================

// --- geofence-search unsupported for: google, tomtom (here, mapbox already tested in cli_integration) ---

#[test]
fn test_geofence_search_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "geofence-search",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_search_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "geofence-search",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// --- geofence-create unsupported for: google, tomtom, mapbox ---

#[test]
fn test_geofence_create_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "geofence-create",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
            "--radius",
            RADIUS_500,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_create_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "geofence-create",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
            "--radius",
            RADIUS_500,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_create_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "geofence-create",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
            "--radius",
            RADIUS_500,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// --- geofence-get unsupported for: google, tomtom, mapbox ---

#[test]
fn test_geofence_get_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "geofence-get",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_get_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "geofence-get",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_get_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "geofence-get",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// --- geofence-delete unsupported for: google, tomtom, mapbox ---

#[test]
fn test_geofence_delete_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "geofence-delete",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_delete_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "geofence-delete",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_geofence_delete_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "geofence-delete",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// ============================================================
// Trip domain (optional, None for non-radar) — 3 commands × remaining providers
// ============================================================

// --- trip-create unsupported for: tomtom, mapbox ---

#[test]
fn test_trip_create_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_create_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// --- trip-update unsupported for: google, tomtom, mapbox ---

#[test]
fn test_trip_update_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "trip-update",
            "--trip-id",
            TEST_TRIP_ID,
            "--status",
            "started",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_update_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "trip-update",
            "--trip-id",
            TEST_TRIP_ID,
            "--status",
            "started",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_update_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "trip-update",
            "--trip-id",
            TEST_TRIP_ID,
            "--status",
            "started",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// --- trip-get unsupported for: google, tomtom, mapbox ---

#[test]
fn test_trip_get_unsupported_for_google() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_get_unsupported_for_tomtom() {
    cli_with_key()
        .args(["--provider", PROVIDER_TOMTOM, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_get_unsupported_for_mapbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_MAPBOX, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
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

#[test]
fn test_radar_supports_geofence_search() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-search",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider").not());
}

#[test]
fn test_radar_supports_trip_create() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider").not());
}

#[test]
fn test_radar_supports_fraud_check() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider").not());
}
