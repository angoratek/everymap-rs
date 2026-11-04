//! Live integration tests for the MapQuest provider.
//!
//! These tests call the public MapQuest platform and require a consumer key.
//! Set `EVERYMAP_MAPQUEST_API_KEY` to run them; when the variable is unset or
//! empty every test returns early, so the default `cargo test` run stays
//! offline and deterministic.
//!
//! The fixture-driven contract tests in this directory cover the wire format;
//! this file checks that the same projections hold against the live services.

use everymap_core::domains::autocomplete::{AutocompleteOptions, AutocompleteProvider};
use everymap_core::domains::matrix::{DistanceMatrixOptions, DistanceMatrixProvider};
use everymap_core::domains::routing::{RouteOptions, Router, TransportMode};
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, PlaceSearchOptions, PlaceSearchProvider, ReverseGeocodeOptions,
};
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::{
    MapQuestAutocompleteProvider, MapQuestClient, MapQuestDistanceMatrix, MapQuestGeocoder,
    MapQuestPlaceSearchProvider, MapQuestProviderConfiguration, MapQuestRasterTileProvider,
    MapQuestRouter, MapQuestTrafficProvider,
};
use std::sync::Arc;

/// The environment variable that carries the consumer key.
const API_KEY_ENVIRONMENT_VARIABLE: &str = "EVERYMAP_MAPQUEST_API_KEY";

/// Denver, the reference location the MapQuest documentation uses.
const DENVER: Coordinate = Coordinate {
    lat: 39.7392,
    lng: -104.9903,
};

/// Boulder, roughly forty kilometres north-west of Denver.
const BOULDER: Coordinate = Coordinate {
    lat: 40.0150,
    lng: -105.2705,
};

/// Builds a client from the environment, or `None` when no key is configured.
fn client_from_environment() -> Option<Arc<MapQuestClient>> {
    let api_key = std::env::var(API_KEY_ENVIRONMENT_VARIABLE).unwrap_or_default();
    if api_key.trim().is_empty() {
        return None;
    }
    Some(Arc::new(MapQuestClient::new(
        MapQuestProviderConfiguration {
            api_key,
            ..Default::default()
        },
    )))
}

#[tokio::test]
async fn live_geocode_denver() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestGeocoder::new(client)
        .geocode("Denver, CO", &GeocodeOptions::default())
        .await
        .expect("the geocoder answers a live request");
    let first = response
        .items
        .first()
        .expect("Denver resolves to at least one location");
    assert!(
        (first.coordinate.lat - DENVER.lat).abs() < 1.0
            && (first.coordinate.lng - DENVER.lng).abs() < 1.0,
        "Denver should resolve near its centre, got {:?}",
        first.coordinate
    );
}

#[tokio::test]
async fn live_reverse_geocode_denver() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestGeocoder::new(client)
        .reverse_geocode(&DENVER, &ReverseGeocodeOptions::default())
        .await
        .expect("the geocoder answers a live reverse request");
    assert!(
        !response.items.is_empty(),
        "a city centre resolves to at least one address"
    );
}

#[tokio::test]
async fn live_route_denver_to_boulder() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestRouter::new(client)
        .calculate_route(
            &DENVER,
            &BOULDER,
            &RouteOptions {
                transport_mode: Some(TransportMode::Car),
                ..Default::default()
            },
        )
        .await
        .expect("the router answers a live request");
    let route = response
        .routes
        .first()
        .expect("the service returns one route");
    assert!(
        route.distance > 10_000.0 && route.distance < 200_000.0,
        "the Denver to Boulder road distance should be tens of kilometres, got {}",
        route.distance
    );
    assert!(!route.steps.is_empty(), "a car route carries steps");
}

#[tokio::test]
async fn live_distance_matrix_two_locations() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestDistanceMatrix::new(client)
        .distance_matrix(
            &[DENVER],
            &[DENVER, BOULDER],
            &DistanceMatrixOptions::default(),
        )
        .await
        .expect("the matrix service answers a live request");
    assert_eq!(
        response.entries.len(),
        2,
        "one origin and two destinations produce two cells"
    );
    let denver_to_denver = response
        .entries
        .iter()
        .find(|entry| entry.destination_index == 0)
        .expect("the Denver cell is present");
    let denver_to_denver_distance = denver_to_denver
        .distance_meters
        .expect("a routed pair reports its distance");
    assert!(
        denver_to_denver_distance < 1_000.0,
        "a location measured against itself stays within a kilometre, got {denver_to_denver_distance}"
    );
}

#[tokio::test]
async fn live_autocomplete_partial_query() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestAutocompleteProvider::new(client)
        .autocomplete(
            "Denver",
            &AutocompleteOptions {
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("the completion service answers a live request");
    assert!(
        !response.items.is_empty(),
        "a city name produces completion candidates"
    );
}

#[tokio::test]
async fn live_place_search_coffee_in_denver() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestPlaceSearchProvider::new(client)
        .search_places(
            "coffee",
            &PlaceSearchOptions {
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("the settled search service answers a live request");
    assert!(
        !response.items.is_empty(),
        "a common business query produces settled results"
    );
}

#[tokio::test]
async fn live_traffic_around_denver() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let response = MapQuestTrafficProvider::new(client)
        .get_traffic(
            &DENVER,
            &TrafficOptions {
                radius: Some(5_000.0),
                ..Default::default()
            },
        )
        .await
        .expect("the traffic service answers a live request");
    assert!(
        !response.flows.is_empty() || !response.incidents.is_empty(),
        "a city centre reports flow or incidents"
    );
}

#[tokio::test]
async fn live_raster_tile_denver() {
    let Some(client) = client_from_environment() else {
        return;
    };
    let tile = MapQuestRasterTileProvider::new(client)
        .fetch_raster_tile(&RasterTileRequest::new(
            3_413,
            6_218,
            14,
            RasterImageFormat::Png,
            RasterTileResolution::Standard,
        ))
        .await
        .expect("the tile service answers a live request");
    assert!(
        !tile.image_bytes.is_empty(),
        "a rendered tile carries an encoded image"
    );
}
