//! Contract tests of the Photon provider against a wiremock server that
//! replays committed fixtures.
//!
//! The harness asserts both directions: the request carries Photon's query
//! parameters (and the identifying user agent), and the GeoJSON
//! `FeatureCollection` maps onto the core's search records, quality scale and
//! autocomplete candidates. Every request targets the in-process server; no
//! test reaches an external service.

use everymap_core::domains::autocomplete::{AutocompleteOptions, AutocompleteProvider};
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, PlaceSearchOptions, PlaceSearchProvider,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_photon::{
    PhotonAutocompleteProvider, PhotonClient, PhotonGeocoder, PhotonPlaceSearchProvider,
    PhotonProviderConfiguration, DEFAULT_PHOTON_USER_AGENT,
};
use std::sync::Arc;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

/// Matches a request whose raw query string does not end with the separator.
///
/// Photon refuses a trailing `&` as a malformed query, so a client that joins
/// parameters by appending the separator fails here before it can fail on the
/// live service.
struct QueryWithoutTrailingSeparator;

impl Match for QueryWithoutTrailingSeparator {
    fn matches(&self, request: &Request) -> bool {
        match request.url.query() {
            Some(raw_query) => !raw_query.ends_with('&'),
            None => true,
        }
    }
}

/// The fixture's first feature's position (Capitol Mall, Sacramento).
const SACRAMENTO_LATITUDE: f64 = 38.579;
const SACRAMENTO_LONGITUDE: f64 = -121.493;

/// Reads one committed fixture.
fn fixture_text(name: &str) -> String {
    let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(fixture_path).expect("the fixture must be readable")
}

/// A client pointed at the in-process server.
fn client_for(server: &MockServer) -> Arc<PhotonClient> {
    Arc::new(PhotonClient::new(PhotonProviderConfiguration {
        base_url: server.uri(),
        user_agent: DEFAULT_PHOTON_USER_AGENT.to_string(),
    }))
}

#[tokio::test]
async fn a_forward_query_maps_features_onto_search_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("q", "Sacramento"))
        .and(query_param("limit", "10"))
        .and(QueryWithoutTrailingSeparator)
        .and(header("user-agent", DEFAULT_PHOTON_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("search_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let response = geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect("the service answers");
    server.verify().await;

    assert!(!response.items.is_empty());
    let first = &response.items[0];
    assert!((first.coordinate.lat - SACRAMENTO_LATITUDE).abs() < 1e-3);
    assert!((first.coordinate.lng - SACRAMENTO_LONGITUDE).abs() < 1e-3);
    assert!(first
        .title
        .as_deref()
        .is_some_and(|title| !title.is_empty()));
    assert!(first.quality.is_some());
}

#[tokio::test]
async fn a_reverse_query_keeps_the_leading_feature() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/reverse"))
        .and(query_param("lat", "38.5816"))
        .and(query_param("lon", "-121.4944"))
        .and(query_param("limit", "1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("reverse_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let response = geocoder
        .reverse_geocode(
            &Coordinate::new(38.5816, -121.4944).expect("the fixture coordinate is valid"),
            &Default::default(),
        )
        .await
        .expect("the service answers");
    server.verify().await;
    assert!(!response.items.is_empty());
}

#[tokio::test]
async fn a_settled_search_carries_the_requested_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("q", "coffee"))
        .and(query_param("limit", "3"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("search_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = PhotonPlaceSearchProvider::new(client_for(&server));
    let response = provider
        .search_places(
            "coffee",
            &PlaceSearchOptions {
                limit: Some(3),
                ..Default::default()
            },
        )
        .await
        .expect("the service answers");
    server.verify().await;
    assert!(!response.items.is_empty());
}

#[tokio::test]
async fn type_ahead_returns_candidates_with_positions() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("q", "Sacra"))
        .and(query_param("limit", "5"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("search_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = PhotonAutocompleteProvider::new(client_for(&server));
    let response = provider
        .autocomplete("Sacra", &AutocompleteOptions::default())
        .await
        .expect("the service answers");
    server.verify().await;
    assert!(!response.items.is_empty());
    assert!(response.items[0].coordinate.is_some());
}

#[tokio::test]
async fn an_empty_query_is_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let error = geocoder
        .geocode("   ", &GeocodeOptions::default())
        .await
        .expect_err("a blank query is refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_missing_features_collection_is_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"type\":\"FeatureCollection\"}"),
        )
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let error = geocoder
        .geocode("Sacramento", &GeocodeOptions::default())
        .await
        .expect_err("a payload without features is refused");
    match error {
        EveryMapError::ProviderError { code, .. } => assert_eq!(code, "MISSING_FEATURES"),
        other => panic!("expected a provider error, got {other:?}"),
    }
}

#[test]
fn the_default_configuration_targets_the_public_instance() {
    let configuration = PhotonProviderConfiguration::default();
    assert_eq!(configuration.base_url, "https://photon.komoot.io");
    assert!(!configuration.user_agent.is_empty());
}
