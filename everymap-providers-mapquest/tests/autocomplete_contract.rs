//! Contract tests of the MapQuest Search Ahead autocomplete provider against a
//! committed fixture.
//!
//! The tests assert the documented query, including the result cap and the
//! ranking centre derived from a preferred bounding box, and the projected
//! candidates.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::autocomplete::{AutocompleteOptions, AutocompleteProvider};
use everymap_core::error::EveryMapError;
use everymap_core::types::{BoundingBox, Coordinate};
use everymap_providers_mapquest::MapQuestAutocompleteProvider;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_recorded_prediction_projects_the_candidates_and_the_result_cap() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/v3/prediction"))
        .and(query_param("q", "denver"))
        .and(query_param(
            "collection",
            "address,adminArea,airport,category,franchise,poi",
        ))
        .and(query_param("limit", "3"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("search_prediction_denver.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestAutocompleteProvider::new(client_for(&server));
    let options = AutocompleteOptions {
        limit: Some(3),
        ..AutocompleteOptions::default()
    };
    let response = provider
        .autocomplete("denver", &options)
        .await
        .expect("the recorded prediction must project");
    server.verify().await;

    assert_eq!(response.items.len(), 3);
    let airport = &response.items[0];
    assert!(airport.title.contains("Denver"));
    assert_eq!(airport.id.as_deref(), Some("mqId:423213150"));
    assert_eq!(airport.categories, vec!["airport".to_string()]);
    let coordinate = airport.coordinate.expect("the candidate has a point");
    assert!((coordinate.lat - 39.85352).abs() < 1e-9);
    assert!((coordinate.lng - (-104.67377)).abs() < 1e-9);
    assert!(airport.raw.is_some());
}

#[tokio::test]
async fn a_preferred_box_becomes_the_ranking_centre() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/v3/prediction"))
        .and(query_param("location", "-104.85,39.75"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("search_prediction_denver.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestAutocompleteProvider::new(client_for(&server));
    let options = AutocompleteOptions {
        limit: Some(5),
        bounding_box: Some(BoundingBox::new(
            Coordinate::new(40.0, -104.5).unwrap(),
            Coordinate::new(39.5, -105.2).unwrap(),
        )),
        ..AutocompleteOptions::default()
    };
    provider
        .autocomplete("denver", &options)
        .await
        .expect("the recorded prediction must project");
    server.verify().await;
}

#[tokio::test]
async fn a_blank_query_is_rejected_without_a_request() {
    let server = MockServer::start().await;
    let provider = MapQuestAutocompleteProvider::new(client_for(&server));
    let error = provider
        .autocomplete("  ", &AutocompleteOptions::default())
        .await
        .expect_err("a blank query must be refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}
