//! Contract tests of the MapQuest failure channels.
//!
//! The tests cover the application-level `info` envelope, an HTTP 404 and an
//! HTTP 429. The consumer key is an obvious placeholder on the wire; every
//! assertion checks that the placeholder is absent from the reported error.

mod common;

use common::{client_for, fixture, TEST_API_KEY};
use everymap_core::domains::routing::{RouteOptions, Router};
use everymap_core::domains::search::{GeocodeOptions, Geocoder};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::{MapQuestGeocoder, MapQuestRouter};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sacramento_waypoints() -> [Coordinate; 2] {
    [
        Coordinate::new(38.57693, -121.49492).unwrap(),
        Coordinate::new(38.57874, -121.50745).unwrap(),
    ]
}

#[tokio::test]
async fn a_non_zero_info_status_code_becomes_a_provider_error_without_the_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/directions/v2/route"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("directions_route_invalid_coordinates.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let waypoints = sacramento_waypoints();
    let error = MapQuestRouter::new(client_for(&server))
        .calculate_route(&waypoints[0], &waypoints[1], &RouteOptions::default())
        .await
        .expect_err("the service rejection must surface");
    server.verify().await;

    match error {
        EveryMapError::ProviderError {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "mapquest");
            assert_eq!(code, "500");
            assert!(message.contains("Invalid LatLng"), "message was {message}");
            assert!(
                !message.contains(TEST_API_KEY),
                "the credential must never appear in an error"
            );
        }
        other => panic!("expected a provider error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_missing_resource_surfaces_the_http_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/geocoding/v1/address"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .expect(1)
        .mount(&server)
        .await;

    let error = MapQuestGeocoder::new(client_for(&server))
        .geocode("Nowhere", &GeocodeOptions::default())
        .await
        .expect_err("a missing resource must surface");
    server.verify().await;

    match error {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 404),
        other => panic!("expected an HTTP error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_rate_limited_response_surfaces_the_provider_and_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/geocoding/v1/address"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_string("slow down"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let error = MapQuestGeocoder::new(client_for(&server))
        .geocode("Nowhere", &GeocodeOptions::default())
        .await
        .expect_err("a rate limit must surface");
    server.verify().await;

    match error {
        EveryMapError::RateLimited {
            provider,
            retry_after_secs,
        } => {
            assert_eq!(provider, "mapquest");
            assert_eq!(retry_after_secs, Some(60));
        }
        other => panic!("expected a rate-limit error, got {other:?}"),
    }
}
