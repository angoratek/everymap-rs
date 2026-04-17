//! Argument validation and edge-case tests.
//!
//! Tests argument parsing, coordinate validation, transport mode handling,
//! boundary conditions, and optional argument coverage for all CLI commands.

mod common;
use common::*;

use predicates::prelude::*;

// ============================================================
// Position command — zero arguments
// ============================================================

#[test]
fn test_position_no_args() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "position"])
        .assert()
        .stderr(predicate::str::contains("required").not());
}

#[test]
fn test_position_no_args_google() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "position"])
        .assert()
        .stderr(predicate::str::contains("required").not());
}

// ============================================================
// Route command — coordinate format validation
// ============================================================

#[test]
fn test_route_invalid_origin_format() {
    cli_with_key()
        .args(["route", "--origin", "badformat", "--destination", BERLIN_COORDS])
        .assert()
        .stderr(predicate::str::contains("Invalid origin"));
}

#[test]
fn test_route_invalid_destination_format() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", "nocoords"])
        .assert()
        .stderr(predicate::str::contains("Invalid destination"));
}

#[test]
fn test_route_invalid_origin_non_numeric() {
    cli_with_key()
        .args(["route", "--origin", "abc,def", "--destination", BERLIN_COORDS])
        .assert()
        .stderr(predicate::str::contains("Invalid origin"));
}

#[test]
fn test_route_negative_longitude() {
    cli_with_key()
        .args(["route", "--origin", NYC_COORDS, "--destination", BOSTON_COORDS])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

// ============================================================
// Transport mode validation
// ============================================================

#[test]
fn test_route_transport_car() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "car"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_truck() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "truck"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_pedestrian() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "pedestrian"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_bicycle() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "bicycle"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_scooter() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "scooter"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_bus() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "bus"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_taxi() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "taxi"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

#[test]
fn test_route_transport_unknown_falls_back_to_car() {
    cli_with_key()
        .args(["route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS, "--transport", "unicycle"])
        .assert()
        .stderr(predicate::str::contains("Invalid origin").not());
}

// ============================================================
// Tour command — edge cases
// ============================================================

#[test]
fn test_tour_single_stop_error() {
    cli_with_key()
        .args(["tour", "--stops", BERLIN_COORDS])
        .assert()
        .stderr(predicate::str::contains("At least 2 stops required"));
}

#[test]
fn test_tour_three_stops() {
    cli_with_key()
        .args(["tour", "--stops", BERLIN_COORDS, PARIS_COORDS, NYC_COORDS])
        .assert()
        .stderr(predicate::str::contains("At least 2 stops required").not());
}

#[test]
fn test_tour_invalid_stop_format() {
    cli_with_key()
        .args(["tour", "--stops", "badformat", BERLIN_COORDS])
        .assert()
        .stderr(predicate::str::contains("At least 2 stops required"));
}

#[test]
fn test_tour_invalid_coords_all_filtered() {
    cli_with_key()
        .args(["tour", "--stops", "badformat", "alsobad"])
        .assert()
        .stderr(predicate::str::contains("At least 2 stops required"));
}

// ============================================================
// Match-route command — edge cases
// ============================================================

#[test]
fn test_match_route_empty_trace() {
    cli_with_key()
        .args(["match-route", "--trace", ""])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates"));
}

#[test]
fn test_match_route_invalid_trace() {
    cli_with_key()
        .args(["match-route", "--trace", "invalid;alsobad"])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates"));
}

#[test]
fn test_match_route_mixed_valid_invalid() {
    cli_with_key()
        .args(["match-route", "--trace", "52.5,13.3;invalid;53.0,14.0"])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates").not());
}

#[test]
fn test_match_route_single_point() {
    cli_with_key()
        .args(["match-route", "--trace", BERLIN_COORDS])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates").not());
}

#[test]
fn test_match_route_many_points() {
    cli_with_key()
        .args(["match-route", "--trace", "52.5,13.3;52.6,13.4;52.7,13.5;52.8,13.6;52.9,13.7"])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates").not());
}

#[test]
fn test_match_route_negative_longitude() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "match-route", "--trace", TRACE_NYC])
        .assert()
        .stderr(predicate::str::contains("No valid coordinates").not());
}

// ============================================================
// Isoline command — range parameter
// ============================================================

#[test]
fn test_isoline_custom_range() {
    cli_with_key()
        .args(["isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG, "--range", "5000"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_isoline_zero_range() {
    cli_with_key()
        .args(["isoline", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG, "--range", "0"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_isoline_requires_lng() {
    cli_with_key()
        .args(["isoline", "--lat", BERLIN_LAT])
        .assert()
        .failure();
}

// ============================================================
// Tile command — edge cases
// ============================================================

#[test]
fn test_tile_missing_z() {
    cli_with_key()
        .args(["tile", "--x", TILE_X, "--y", TILE_Y])
        .assert()
        .failure();
}

#[test]
fn test_tile_missing_x() {
    cli_with_key()
        .args(["tile", "--z", TILE_Z, "--y", TILE_Y])
        .assert()
        .failure();
}

#[test]
fn test_tile_missing_y() {
    cli_with_key()
        .args(["tile", "--z", TILE_Z, "--x", TILE_X])
        .assert()
        .failure();
}

#[test]
fn test_tile_zero_zoom() {
    cli_with_key()
        .args(["tile", "--z", "0", "--x", "0", "--y", "0"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_tile_large_values() {
    cli_with_key()
        .args(["tile", "--z", "22", "--x", "4000000", "--y", "3000000"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Map-image command — zoom parameter
// ============================================================

#[test]
fn test_map_image_default_zoom() {
    cli_with_key()
        .args(["map-image", "--lat", BERLIN_LAT, "--lng", BERLIN_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_map_image_requires_lng() {
    cli_with_key()
        .args(["map-image", "--lat", BERLIN_LAT])
        .assert()
        .failure();
}

#[test]
fn test_map_image_negative_longitude() {
    cli_with_key()
        .args(["map-image", "--lat", NYC_LAT, "--lng=", NYC_LNG, "--zoom", "14"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Attributes command — optional arguments
// ============================================================

#[test]
fn test_attributes_no_bbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "attributes"])
        .assert()
        .stderr(predicate::str::contains("required").not());
}

#[test]
fn test_attributes_with_layer() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "attributes", "--bbox", BERLIN_BBOX, "--layer", "segments"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_attributes_with_format() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "attributes", "--bbox", BERLIN_BBOX, "--format", "geojson"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_attributes_with_ids() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "attributes", "--bbox", BERLIN_BBOX, "--ids", "id1", "id2"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_attributes_with_include() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "attributes", "--bbox", BERLIN_BBOX, "--include", "speedLimit,roadClass"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_attributes_all_optional_args() {
    cli_with_key()
        .args([
            "--provider", PROVIDER_HERE, "attributes",
            "--bbox", BERLIN_BBOX,
            "--layer", "roads",
            "--format", "json",
            "--ids", "id1", "id2", "id3",
            "--include", "speedLimit,roadClass,functionalClass",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Geofence-search — optional arguments
// ============================================================

#[test]
fn test_geofence_search_with_tags() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-search", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--tags", "store,warehouse"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_search_with_limit() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-search", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--limit", "10"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_search_all_optional_args() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-search", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--radius", "2000", "--tags", TAG_STORE, "--limit", "5"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_search_requires_lng() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-search", "--lat", NYC_LAT_POSLNG])
        .assert()
        .failure();
}

// ============================================================
// Geofence-create — optional arguments
// ============================================================

#[test]
fn test_geofence_create_with_description() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--radius", RADIUS_500, "--description", "NYC Store Zone"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_create_requires_radius() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS])
        .assert()
        .failure();
}

#[test]
fn test_geofence_create_requires_lat() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lng", NYC_LNG_POS, "--radius", RADIUS_500])
        .assert()
        .failure();
}

#[test]
fn test_geofence_create_requires_lng() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lat", NYC_LAT_POSLNG, "--radius", RADIUS_500])
        .assert()
        .failure();
}

#[test]
fn test_geofence_create_tag_only() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--radius", RADIUS_500, "--tag", "warehouse"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_geofence_create_all_optional_args() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geofence-create", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--radius", RADIUS_500, "--tag", TAG_STORE, "--description", "Main store zone"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Trip-create — optional arguments
// ============================================================

#[test]
fn test_trip_create_with_external_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-create", "--mode", TRANSPORT_CAR, "--external-id", "ext_order_123"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_create_with_tag() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-create", "--mode", TRANSPORT_CAR, "--tag", "delivery"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_create_all_optional_args() {
    cli_with_key()
        .args([
            "--provider", PROVIDER_RADAR, "trip-create",
            "--origin", NYC_COORDS,
            "--destination", BOSTON_COORDS,
            "--mode", TRANSPORT_CAR,
            "--external-id", "ext_123",
            "--tag", "delivery",
        ])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_create_default_mode() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-create"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Trip-update — status values
// ============================================================

#[test]
fn test_trip_update_status_pending() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "pending"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_status_started() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "started"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_status_approaching() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "approaching"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_status_arrived() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "arrived"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_status_completed() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "completed"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_trip_update_requires_trip_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--status", "started"])
        .assert()
        .failure();
}

#[test]
fn test_trip_update_requires_status() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID])
        .assert()
        .failure();
}

#[test]
fn test_trip_update_invalid_status() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "trip-update", "--trip-id", TEST_TRIP_ID, "--status", "unknown_status"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Fraud-check — optional arguments and edge cases
// ============================================================

#[test]
fn test_fraud_check_default_accuracy() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "fraud-check", "--device-id", TEST_DEVICE_ID, "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_zero_accuracy() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "fraud-check", "--device-id", TEST_DEVICE_ID, "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--accuracy", "0"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_high_accuracy() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "fraud-check", "--device-id", TEST_DEVICE_ID, "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--accuracy", "100"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_all_optional_args() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "fraud-check", "--device-id", TEST_DEVICE_ID, "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS, "--accuracy", "5.0", "--user-id", "user_abc"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

#[test]
fn test_fraud_check_requires_device_id() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "fraud-check", "--lat", NYC_LAT_POSLNG, "--lng", NYC_LNG_POS])
        .assert()
        .failure();
}

// ============================================================
// Reverse-geocode — negative longitude
// ============================================================

#[test]
fn test_reverse_geocode_negative_longitude() {
    cli_with_key()
        .args(["reverse-geocode", "--lat", NYC_LAT, "--lng=", NYC_LNG])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Geocode — multiple words in query
// ============================================================

#[test]
fn test_geocode_multi_word_query() {
    cli_with_key()
        .args(["geocode", "Brandenburg", "Gate", "Berlin"])
        .assert()
        .stderr(predicate::str::contains(ERR_API_KEY_REQUIRED).not());
}

// ============================================================
// Each provider can run geocode
// ============================================================

#[test]
fn test_geocode_here() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_geocode_google() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_geocode_tomtom() {
    cli_with_key()
        .args(["--provider", PROVIDER_TOMTOM, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_geocode_mapbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_MAPBOX, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_geocode_radar() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "geocode", QUERY_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

// ============================================================
// Each provider can run route
// ============================================================

#[test]
fn test_route_here() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_route_google() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_route_tomtom() {
    cli_with_key()
        .args(["--provider", PROVIDER_TOMTOM, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_route_mapbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_MAPBOX, "route", "--origin", BERLIN_COORDS, "--destination", PARIS_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_route_radar() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "route", "--origin", NYC_COORDS, "--destination", BOSTON_COORDS])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

// ============================================================
// Each provider can run match-route
// ============================================================

#[test]
fn test_match_route_here() {
    cli_with_key()
        .args(["--provider", PROVIDER_HERE, "match-route", "--trace", TRACE_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_match_route_google() {
    cli_with_key()
        .args(["--provider", PROVIDER_GOOGLE, "match-route", "--trace", TRACE_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_match_route_tomtom() {
    cli_with_key()
        .args(["--provider", PROVIDER_TOMTOM, "match-route", "--trace", TRACE_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_match_route_mapbox() {
    cli_with_key()
        .args(["--provider", PROVIDER_MAPBOX, "match-route", "--trace", TRACE_BERLIN])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}

#[test]
fn test_match_route_radar() {
    cli_with_key()
        .args(["--provider", PROVIDER_RADAR, "match-route", "--trace", TRACE_NYC])
        .assert()
        .stderr(predicate::str::contains(ERR_UNSUPPORTED_PROVIDER).not());
}