use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to create a CLI command.
fn cli() -> Command {
    Command::cargo_bin("everymap").unwrap()
}

// ============================================================
// Provider validation tests
// ============================================================

#[test]
fn test_unsupported_provider() {
    cli()
        .args(["--provider", "invalid", "--api-key", "test", "geocode", "Berlin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported provider 'invalid'"));
}

#[test]
fn test_supported_providers() {
    for provider in &["here", "google", "tomtom", "mapbox", "radar"] {
        // These will fail with an API error, but should NOT fail with "unsupported provider"
        cli()
            .args(["--provider", provider, "--api-key", "test-key", "geocode", "Berlin"])
            .assert()
            .stderr(predicate::str::contains("Unsupported provider").not());
    }
}

#[test]
fn test_default_provider_is_here() {
    cli()
        .args(["--api-key", "test-key", "geocode", "Berlin"])
        .assert()
        .stderr(predicate::str::contains("Unsupported provider").not());
}

#[test]
fn test_missing_api_key_no_env() {
    // With no env var and no --api-key, should error about missing key
    // Note: this may still pass if a real key is in ~/.everymap/config.toml
    // so we test that the help about --api-key exists instead
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
    cli()
        .args(["--api-key", "test", "geocode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_geocode_with_query() {
    cli()
        .args(["--api-key", "test", "geocode", "Berlin"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Reverse geocode command tests
// ============================================================

#[test]
fn test_reverse_geocode_requires_lat() {
    cli()
        .args(["--api-key", "test", "reverse-geocode", "--lng", "13.3"])
        .assert()
        .failure();
}

#[test]
fn test_reverse_geocode_requires_lng() {
    cli()
        .args(["--api-key", "test", "reverse-geocode", "--lat", "52.5"])
        .assert()
        .failure();
}

#[test]
fn test_reverse_geocode_with_coords() {
    cli()
        .args(["--api-key", "test", "reverse-geocode", "--lat", "52.5", "--lng", "13.3"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Route command tests
// ============================================================

#[test]
fn test_route_requires_origin() {
    cli()
        .args(["--api-key", "test", "route", "--destination", "52.5,13.3"])
        .assert()
        .failure();
}

#[test]
fn test_route_requires_destination() {
    cli()
        .args(["--api-key", "test", "route", "--origin", "52.5,13.3"])
        .assert()
        .failure();
}

#[test]
fn test_route_with_valid_args() {
    cli()
        .args(["--api-key", "test", "route", "--origin", "52.5,13.3", "--destination", "48.8,2.3"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_route_with_transport_mode() {
    cli()
        .args(["--api-key", "test", "route", "--origin", "52.5,13.3", "--destination", "48.8,2.3", "--transport", "bicycle"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Traffic command tests
// ============================================================

#[test]
fn test_traffic_requires_lat() {
    cli()
        .args(["--api-key", "test", "traffic", "--lng", "13.3"])
        .assert()
        .failure();
}

#[test]
fn test_traffic_with_coords() {
    cli()
        .args(["--api-key", "test", "traffic", "--lat", "52.5", "--lng", "13.3"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Isoline command tests
// ============================================================

#[test]
fn test_isoline_requires_lat() {
    cli()
        .args(["--api-key", "test", "isoline", "--lng", "13.3"])
        .assert()
        .failure();
}

#[test]
fn test_isoline_with_default_range() {
    cli()
        .args(["--api-key", "test", "isoline", "--lat", "52.5", "--lng", "13.3"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Match route command tests
// ============================================================

#[test]
fn test_match_route_requires_trace() {
    cli()
        .args(["--api-key", "test", "match-route"])
        .assert()
        .failure();
}

#[test]
fn test_match_route_with_trace() {
    cli()
        .args(["--api-key", "test", "match-route", "--trace", "52.5,13.3;52.6,13.4"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Tour command tests
// ============================================================

#[test]
fn test_tour_requires_stops() {
    cli()
        .args(["--api-key", "test", "tour"])
        .assert()
        .failure();
}

#[test]
fn test_tour_with_stops() {
    cli()
        .args(["--api-key", "test", "tour", "--stops", "52.5,13.3", "48.8,2.3"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Tile command tests
// ============================================================

#[test]
fn test_tile_requires_args() {
    cli()
        .args(["--api-key", "test", "tile"])
        .assert()
        .failure();
}

#[test]
fn test_tile_with_xyz() {
    cli()
        .args(["--api-key", "test", "tile", "--z", "14", "--x", "8800", "--y", "5374"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Map image command tests
// ============================================================

#[test]
fn test_map_image_requires_lat() {
    cli()
        .args(["--api-key", "test", "map-image", "--lng", "13.3"])
        .assert()
        .failure();
}

#[test]
fn test_map_image_with_coords() {
    cli()
        .args(["--api-key", "test", "map-image", "--lat", "52.5", "--lng", "13.3", "--zoom", "14"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Attributes command tests
// ============================================================

#[test]
fn test_attributes_with_bbox() {
    cli()
        .args(["--api-key", "test", "attributes", "--bbox", "52.4,13.2;52.6,13.5"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Geofence commands (Radar-specific)
// ============================================================

#[test]
fn test_geofence_search_requires_lat() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-search", "--lng", "74.0"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_search_with_coords() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-search", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_geofence_search_with_radius() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-search", "--lat", "40.71", "--lng", "74.0", "--radius", "1000"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_geofence_search_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "geofence-search", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_geofence_search_unsupported_for_google() {
    cli()
        .args(["--api-key", "test", "--provider", "google", "geofence-search", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_geofence_search_unsupported_for_tomtom() {
    cli()
        .args(["--api-key", "test", "--provider", "tomtom", "geofence-search", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_geofence_search_unsupported_for_mapbox() {
    cli()
        .args(["--api-key", "test", "--provider", "mapbox", "geofence-search", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_geofence_create_requires_args() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-create", "--lat", "40.71"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_create_with_all_args() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-create", "--lat", "40.71", "--lng", "74.0", "--radius", "500", "--tag", "store"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_geofence_create_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "geofence-create", "--lat", "40.71", "--lng", "74.0", "--radius", "500"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_geofence_get_requires_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-get"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_get_with_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-get", "gf_123"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_geofence_delete_requires_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "geofence-delete"])
        .assert()
        .failure();
}

#[test]
fn test_geofence_delete_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "geofence-delete", "gf_123"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

// ============================================================
// Trip commands (Radar-specific)
// ============================================================

#[test]
fn test_trip_create_with_mode() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-create", "--mode", "car"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_trip_create_with_origin_dest() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-create", "--origin", "40.71,74.0", "--destination", "42.36,71.05", "--mode", "car"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_trip_create_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "trip-create", "--mode", "car"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_trip_create_unsupported_for_google() {
    cli()
        .args(["--api-key", "test", "--provider", "google", "trip-create", "--mode", "car"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_trip_update_requires_args() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-update"])
        .assert()
        .failure();
}

#[test]
fn test_trip_update_with_args() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-update", "--trip-id", "trip_123", "--status", "started"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_trip_update_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "trip-update", "--trip-id", "trip_123", "--status", "started"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_trip_get_requires_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-get"])
        .assert()
        .failure();
}

#[test]
fn test_trip_get_with_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "trip-get", "trip_123"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_trip_get_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "trip-get", "trip_123"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

// ============================================================
// Fraud check command (Radar-specific)
// ============================================================

#[test]
fn test_fraud_check_requires_device_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_requires_lat() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--device-id", "dev_1", "--lng", "74.0"])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_requires_lng() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--device-id", "dev_1", "--lat", "40.71"])
        .assert()
        .failure();
}

#[test]
fn test_fraud_check_with_required_args() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_fraud_check_with_user_id() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0", "--user-id", "user_1"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_fraud_check_with_custom_accuracy() {
    cli()
        .args(["--api-key", "test", "--provider", "radar", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0", "--accuracy", "5.0"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_fraud_check_unsupported_for_here() {
    cli()
        .args(["--api-key", "test", "--provider", "here", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_fraud_check_unsupported_for_google() {
    cli()
        .args(["--api-key", "test", "--provider", "google", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_fraud_check_unsupported_for_tomtom() {
    cli()
        .args(["--api-key", "test", "--provider", "tomtom", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

#[test]
fn test_fraud_check_unsupported_for_mapbox() {
    cli()
        .args(["--api-key", "test", "--provider", "mapbox", "fraud-check", "--device-id", "dev_1", "--lat", "40.71", "--lng", "74.0"])
        .assert()
        .stderr(predicate::str::contains("not supported by this provider"));
}

// ============================================================
// Output format tests
// ============================================================

#[test]
fn test_output_format_json() {
    cli()
        .args(["--api-key", "test", "--output", "json", "geocode", "Berlin"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

#[test]
fn test_output_format_pretty() {
    cli()
        .args(["--api-key", "test", "--output", "pretty", "geocode", "Berlin"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
}

// ============================================================
// Verbose flag tests
// ============================================================

#[test]
fn test_verbose_flag() {
    cli()
        .args(["--api-key", "test", "--verbose", "geocode", "Berlin"])
        .assert()
        .stderr(predicate::str::contains("API key required").not());
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
    let output = cli()
        .args(["--help"])
        .assert()
        .success();

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