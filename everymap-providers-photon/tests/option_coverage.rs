//! Contract tests for the Photon options the API can and cannot express.
//!
//! Photon serves a result cap, a language, a bias point and a bounding box.
//! An option it cannot express is refused before any request, so a caller
//! never believes a constraint was applied. Every request targets the
//! in-process server; no test reaches an external service.

use everymap_core::domains::autocomplete::{AutocompleteOptions, AutocompleteProvider};
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResultType,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::{BoundingBox, Coordinate};
use everymap_providers_photon::{
    PhotonAutocompleteProvider, PhotonClient, PhotonGeocoder, PhotonProviderConfiguration,
    DEFAULT_PHOTON_USER_AGENT,
};
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
async fn a_forward_language_reaches_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("q", "Sacramento"))
        .and(query_param("lang", "de"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("search_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    geocoder
        .geocode(
            "Sacramento",
            &GeocodeOptions {
                language: Some("de".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("the service answers");
    server.verify().await;
}

#[tokio::test]
async fn a_bounding_box_biases_the_query_towards_its_centre() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("lat", "52.5"))
        .and(query_param("lon", "13.25"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("search_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    geocoder
        .geocode(
            "Sacramento",
            &GeocodeOptions {
                bounding_box: Some(BoundingBox {
                    south_west: Coordinate::new(52.0, 13.0).expect("valid coordinate"),
                    north_east: Coordinate::new(53.0, 13.5).expect("valid coordinate"),
                }),
                ..Default::default()
            },
        )
        .await
        .expect("the service answers");
    server.verify().await;
}

#[tokio::test]
async fn a_country_code_filter_is_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let error = geocoder
        .geocode(
            "Sacramento",
            &GeocodeOptions {
                country_codes: vec!["USA".to_string(), "CAN".to_string()],
                ..Default::default()
            },
        )
        .await
        .expect_err("a country filter Photon cannot express must be refused");
    server.verify().await;
    match error {
        EveryMapError::ValidationError(detail) => {
            assert!(detail.contains("USA"), "{detail}");
            assert!(detail.contains("CAN"), "{detail}");
        }
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_type_ahead_country_code_filter_is_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let provider = PhotonAutocompleteProvider::new(client_for(&server));
    let error = provider
        .autocomplete(
            "Sacra",
            &AutocompleteOptions {
                country_codes: vec!["USA".to_string()],
                ..Default::default()
            },
        )
        .await
        .expect_err("a country filter Photon cannot express must be refused");
    server.verify().await;
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_reverse_radius_is_refused_before_any_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let error = geocoder
        .reverse_geocode(
            &Coordinate::new(38.5816, -121.4944).expect("the coordinate is valid"),
            &ReverseGeocodeOptions {
                radius: Some(500.0),
                ..Default::default()
            },
        )
        .await
        .expect_err("a reverse radius Photon cannot express must be refused");
    server.verify().await;
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_reverse_language_reaches_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/reverse"))
        .and(query_param("lat", "38.5816"))
        .and(query_param("lon", "-121.4944"))
        .and(query_param("limit", "1"))
        .and(query_param("lang", "fr"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("reverse_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let response = geocoder
        .reverse_geocode(
            &Coordinate::new(38.5816, -121.4944).expect("the coordinate is valid"),
            &ReverseGeocodeOptions {
                language: Some("fr".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("the service answers");
    server.verify().await;
    assert!(!response.items.is_empty());
}

#[tokio::test]
async fn a_reverse_limit_reaches_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/reverse"))
        .and(query_param("limit", "3"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture_text("reverse_sacramento.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = PhotonGeocoder::new(client_for(&server));
    let response = geocoder
        .reverse_geocode(
            &Coordinate::new(38.5816, -121.4944).expect("the coordinate is valid"),
            &ReverseGeocodeOptions {
                limit: Some(3),
                ..Default::default()
            },
        )
        .await
        .expect("the service answers");
    server.verify().await;
    assert_eq!(response.items[0].result_type, SearchResultType::ExactMatch);
}
