//! Contract tests of the MapQuest distance matrix against a committed
//! fixture.
//!
//! The tests assert the POST body carries the union of origins and
//! destinations with the all-to-all option, and that the square reply is
//! filtered down to the wanted cells in row-major origin-major order.

mod common;

use common::{client_for, fixture, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::matrix::{
    DistanceMatrixOptions, DistanceMatrixProvider, DistanceMatrixResponse,
};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_mapquest::MapQuestDistanceMatrix;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn coordinate(latitude: f64, longitude: f64) -> Coordinate {
    Coordinate::new(latitude, longitude).unwrap()
}

#[tokio::test]
async fn a_recorded_matrix_projects_the_wanted_pair_measurements() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/directions/v2/routematrix"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .and(body_json(json!({
            "locations": [
                {"latLng": {"lat": 38.57693, "lng": -121.49492}},
                {"latLng": {"lat": 38.57874, "lng": -121.50745}},
                {"latLng": {"lat": 38.57393, "lng": -121.46956}}
            ],
            "options": {
                "allToAll": true,
                "unit": "k",
                "routeType": "fastest"
            }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("routematrix_three_locations_unit_k.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let matrix = MapQuestDistanceMatrix::new(client_for(&server));
    let origins = [coordinate(38.57693, -121.49492)];
    let destinations = [
        coordinate(38.57874, -121.50745),
        coordinate(38.57393, -121.46956),
    ];
    let response: DistanceMatrixResponse = matrix
        .distance_matrix(&origins, &destinations, &DistanceMatrixOptions::default())
        .await
        .expect("the recorded matrix must project");
    server.verify().await;

    assert_eq!(response.entries.len(), 2);
    let first = &response.entries[0];
    assert_eq!(first.origin_index, 0);
    assert_eq!(first.destination_index, 0);
    assert!((first.distance_meters.unwrap() - 1700.0).abs() < 100.0);
    assert!((first.duration_seconds.unwrap() - 235.0).abs() < 1.0);
    let second = &response.entries[1];
    assert_eq!(second.origin_index, 0);
    assert_eq!(second.destination_index, 1);
    assert!((second.distance_meters.unwrap() - 2600.0).abs() < 100.0);
    assert!((second.duration_seconds.unwrap() - 320.0).abs() < 1.0);
}

#[tokio::test]
async fn an_empty_list_is_rejected_before_any_request() {
    let server = MockServer::start().await;
    let matrix = MapQuestDistanceMatrix::new(client_for(&server));
    let error = matrix
        .distance_matrix(
            &[],
            &[coordinate(38.0, -121.0)],
            &DistanceMatrixOptions::default(),
        )
        .await
        .expect_err("an empty origin list must be refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));

    let error = matrix
        .distance_matrix(
            &[coordinate(38.0, -121.0)],
            &[],
            &DistanceMatrixOptions::default(),
        )
        .await
        .expect_err("an empty destination list must be refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}
