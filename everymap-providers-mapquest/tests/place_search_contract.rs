//! Contract tests of the MapQuest settled place search against a committed
//! fixture.
//!
//! The tests assert the documented query, the settled result cap and the
//! projection of prediction features into search results with their
//! granularity.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::search::{PlaceSearchOptions, PlaceSearchProvider};
use everymap_providers_mapquest::MapQuestPlaceSearchProvider;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_settled_search_asks_for_the_documented_cap_and_projects_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/v3/prediction"))
        .and(query_param("q", "denver"))
        .and(query_param(
            "collection",
            "address,adminArea,airport,category,franchise,poi",
        ))
        .and(query_param("limit", "15"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("search_prediction_denver.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestPlaceSearchProvider::new(client_for(&server));
    let response = provider
        .search_places("denver", &PlaceSearchOptions::default())
        .await
        .expect("the recorded prediction must project");
    assert!(!response.items.is_empty());
    assert!(response.items.iter().all(|item| {
        item.title.as_deref().is_some_and(|title| !title.is_empty())
            && item.address.label.as_deref() == item.title.as_deref()
    }));
    let first = &response.items[0];
    assert!(
        first.quality.is_some(),
        "the recorded feature's collection names its granularity"
    );
    server.verify().await;
}

#[tokio::test]
async fn an_empty_settled_query_is_refused_before_any_request() {
    let server = MockServer::start().await;
    let provider = MapQuestPlaceSearchProvider::new(client_for(&server));
    let error = provider
        .search_places("   ", &PlaceSearchOptions::default())
        .await
        .expect_err("an empty query has nothing to search for");
    assert!(matches!(
        error,
        everymap_core::error::EveryMapError::ValidationError(_)
    ));
}
