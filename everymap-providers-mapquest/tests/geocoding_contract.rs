//! Contract tests of the MapQuest geocoder against committed fixtures.
//!
//! The tests assert both directions: the request carries the documented path,
//! query parameters, credential and user agent, and the recorded answer
//! projects into the core's `SearchResponse` with the expected quality and
//! address fields.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::search::{
    GeocodeOptions, Geocoder, ReverseGeocodeOptions, SearchResultType,
};
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::MapQuestGeocoder;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn forward_geocode_projects_the_recorded_city_location() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/geocoding/v1/address"))
        .and(query_param("location", "Sacramento,California"))
        .and(query_param("maxResults", "2"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("geocode_address_city.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = MapQuestGeocoder::new(client_for(&server));
    let options = GeocodeOptions {
        limit: Some(2),
        ..GeocodeOptions::default()
    };
    let response = geocoder
        .geocode("Sacramento,California", &options)
        .await
        .expect("the recorded geocode must project");
    server.verify().await;

    assert_eq!(response.items.len(), 1);
    let result = &response.items[0];
    assert_eq!(result.coordinate.lat, 38.57944);
    assert_eq!(result.coordinate.lng, -121.49085);
    assert_eq!(result.result_type, SearchResultType::Interpolated);
    assert_eq!(result.address.city.as_deref(), Some("Sacramento"));
    assert_eq!(result.address.state_code.as_deref(), Some("CA"));
    assert_eq!(result.address.country_code.as_deref(), Some("US"));
    assert_eq!(result.address.label.as_deref(), Some("Sacramento, CA, US"));
    assert!(result.raw.is_some());
}

#[tokio::test]
async fn reverse_geocode_projects_the_recorded_point_location() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/geocoding/v1/reverse"))
        .and(query_param("location", "38.5766,-121.4944"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("geocode_reverse_capitol.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let geocoder = MapQuestGeocoder::new(client_for(&server));
    let response = geocoder
        .reverse_geocode(
            &Coordinate::new(38.5766, -121.4944).unwrap(),
            &ReverseGeocodeOptions::default(),
        )
        .await
        .expect("the recorded reverse geocode must project");
    server.verify().await;

    assert_eq!(response.items.len(), 1);
    let result = &response.items[0];
    assert_eq!(result.result_type, SearchResultType::ExactMatch);
    assert_eq!(result.address.street.as_deref(), Some("1315 10th St"));
    assert_eq!(result.address.city.as_deref(), Some("Sacramento"));
    assert_eq!(result.address.postal_code.as_deref(), Some("95814"));
    assert_eq!(result.address.district.as_deref(), Some("Capitol Mall"));
}

#[tokio::test]
async fn a_blank_query_is_rejected_without_a_request() {
    let server = MockServer::start().await;
    let geocoder = MapQuestGeocoder::new(client_for(&server));
    let error = geocoder
        .geocode("   ", &GeocodeOptions::default())
        .await
        .expect_err("a blank query must be refused");
    assert!(matches!(
        error,
        everymap_core::error::EveryMapError::ValidationError(_)
    ));
}
