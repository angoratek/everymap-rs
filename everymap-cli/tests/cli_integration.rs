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
// Geofence commands (Radar-specific)
// ============================================================

#[test]
fn test_geofence_search_requires_lat() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-search",
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .failure();
}

#[test]
fn test_geofence_search_with_coords() {
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
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_search_with_radius() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-search",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
            "--radius",
            RADIUS_1000,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_search_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
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

#[test]
fn test_geofence_search_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
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
fn test_geofence_create_requires_args() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-create",
            "--lat",
            NYC_LAT_POSLNG,
        ])
        .assert()
        .failure();
}

#[test]
fn test_geofence_create_with_all_args() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-create",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
            "--radius",
            RADIUS_500,
            "--tag",
            TAG_STORE,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_create_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
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
fn test_geofence_get_requires_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-get"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_get_with_id() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "geofence-get",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_delete_requires_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-delete"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_delete_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
            "geofence-delete",
            TEST_GEOFENCE_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// ============================================================
// Trip commands (Radar-specific)
// ============================================================

#[test]
fn test_trip_create_with_mode() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_create_with_origin_dest() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "trip-create",
            "--origin",
            NYC_COORDS,
            "--destination",
            BOSTON_COORDS,
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_create_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_create_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "trip-create",
            "--mode",
            TRANSPORT_CAR,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_trip_update_requires_args() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update"])
        .assert()
        .failure();
}

#[test]
fn test_trip_update_with_args() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "trip-update",
            "--trip-id",
            TEST_TRIP_ID,
            "--status",
            "started",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
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
fn test_trip_get_requires_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-get"])
        .assert()
        .failure();
}

#[test]
fn test_trip_get_with_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_get_unsupported_for_here() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "trip-get", TEST_TRIP_ID])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

// ============================================================
// Fraud check command (Radar-specific)
// ============================================================

#[test]
fn test_fraud_check_requires_device_id() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "fraud-check",
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_requires_lat() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_requires_lng() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_RADAR,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
        ])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_with_required_args() {
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
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_with_user_id() {
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
            "--user-id",
            TEST_USER_ID,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_with_custom_accuracy() {
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
            "--accuracy",
            "5.0",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_unsupported_for_here() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_HERE,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_fraud_check_unsupported_for_google() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_GOOGLE,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_fraud_check_unsupported_for_tomtom() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_TOMTOM,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
}

#[test]
fn test_fraud_check_unsupported_for_mapbox() {
    cli_with_key()
        .args([
            "--provider",
            PROVIDER_MAPBOX,
            "fraud-check",
            "--device-id",
            TEST_DEVICE_ID,
            "--lat",
            NYC_LAT_POSLNG,
            "--lng",
            NYC_LNG_POS,
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_NOT_SUPPORTED));
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
    assert!(stdout.contains("geofence-search"));
    assert!(stdout.contains("geofence-create"));
    assert!(stdout.contains("geofence-get"));
    assert!(stdout.contains("geofence-delete"));
    assert!(stdout.contains("trip-create"));
    assert!(stdout.contains("trip-update"));
    assert!(stdout.contains("trip-get"));
    assert!(stdout.contains("fraud-check"));
}
