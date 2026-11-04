//! Distance matrices through the MapQuest Directions API v2 route matrix
//! endpoint.
//!
//! MapQuest computes all-to-all matrices only. The provider therefore sends
//! the union of the request's origins and destinations as one `locations`
//! array, asks for an all-to-all matrix, and filters the wanted
//! origin-destination cells out of the square reply. The core contract's
//! row-major origin-major ordering and its `usize` indices are preserved;
//! unroutable pairs reported as `null` are omitted.
//!
//! Distances are requested in kilometres and converted to meters, and
//! durations are returned in seconds.

use crate::client::MapQuestClient;
use crate::error_mapping::invalid_response;
use crate::json_coordinates::coordinate_to_request_value;
use crate::routing::{effective_transport_mode, route_type_query_text};
use async_trait::async_trait;
use everymap_core::domains::matrix::{
    DistanceMatrixEntry, DistanceMatrixOptions, DistanceMatrixProvider, DistanceMatrixResponse,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use serde_json::{json, Value};
use std::sync::Arc;

/// Service path of the route matrix endpoint.
const ROUTE_MATRIX_PATH: &str = "directions/v2/routematrix";

/// Unit value requesting kilometres; the projection converts them to meters.
const METRIC_UNIT_VALUE: &str = "k";

/// Body key naming the requested locations.
const LOCATIONS_KEY: &str = "locations";

/// Body key naming the matrix options.
const OPTIONS_KEY: &str = "options";

/// Option key requesting an all-to-all matrix.
const ALL_TO_ALL_KEY: &str = "allToAll";

/// Option key selecting the distance unit.
const UNIT_KEY: &str = "unit";

/// Option key selecting the route cost model.
const ROUTE_TYPE_KEY: &str = "routeType";

/// Meters in one kilometre.
const METERS_PER_KILOMETER: f64 = 1000.0;

/// Distance matrices through the MapQuest route matrix endpoint.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`].
pub struct MapQuestDistanceMatrix {
    client: Arc<MapQuestClient>,
}

impl MapQuestDistanceMatrix {
    /// Builds a distance-matrix provider around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A provider ready to measure origin-destination pairs.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DistanceMatrixProvider for MapQuestDistanceMatrix {
    /// Calculates durations and distances for every origin-destination pair.
    ///
    /// # Parameters
    /// - `origins` — the origin coordinates.
    /// - `destinations` — the destination coordinates.
    /// - `options` — the transport mode and strategy. MapQuest measures the
    ///   matrix for automobile, bicycle or pedestrian routing; an unsupported
    ///   mode is measured as an automobile with a log warning.
    ///
    /// # Returns
    /// One entry per routable pair, in row-major origin-major order. Pairs the
    /// service reports as unroutable are omitted.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when either list is empty.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when the body is not JSON.
    /// - [`EveryMapError::ProviderError`] when the service reports a non-zero
    ///   `info.statuscode`, the reply lacks `distance` or `time`, or the reply
    ///   does not match the requested location count.
    async fn distance_matrix(
        &self,
        origins: &[Coordinate],
        destinations: &[Coordinate],
        options: &DistanceMatrixOptions,
    ) -> EveryMapResult<DistanceMatrixResponse> {
        if origins.is_empty() || destinations.is_empty() {
            return Err(EveryMapError::ValidationError(
                "a distance matrix needs at least one origin and one destination".to_string(),
            ));
        }
        if options.provider_extra.is_some() {
            log::warn!("MapQuest distance matrices do not consume provider_extra; ignoring");
        }
        let transport_mode = effective_transport_mode(options.transport_mode);
        let locations = origins
            .iter()
            .chain(destinations.iter())
            .map(coordinate_to_request_value)
            .collect::<Vec<Value>>();
        let request_body = json!({
            LOCATIONS_KEY: locations,
            OPTIONS_KEY: {
                ALL_TO_ALL_KEY: true,
                UNIT_KEY: METRIC_UNIT_VALUE,
                ROUTE_TYPE_KEY: route_type_query_text(transport_mode, options.route_strategy),
            }
        });
        let payload = self
            .client
            .post_json(ROUTE_MATRIX_PATH, &[], &request_body)
            .await?;
        parse_matrix_payload(&payload, origins.len(), destinations.len())
    }
}

/// Projects a route-matrix reply into the core response.
///
/// `distance` and `time` are square row-major arrays over the concatenated
/// origin-then-destination location list. The projection keeps the origin rows
/// and the destination columns the caller asked for, converts kilometres to
/// meters, and skips a cell when either value is `null`.
///
/// # Parameters
/// - `payload` — a decoded route-matrix response body.
/// - `origin_count` — how many leading locations are origins.
/// - `destination_count` — how many trailing locations are destinations.
///
/// # Returns
/// One entry per measured pair, in row-major origin-major order.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] when `distance` or `time` is
/// missing, is not nested arrays, or does not match the requested location
/// count.
pub fn parse_matrix_payload(
    payload: &Value,
    origin_count: usize,
    destination_count: usize,
) -> EveryMapResult<DistanceMatrixResponse> {
    let total_location_count = origin_count + destination_count;
    let distances = payload
        .get("distance")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("matrix payload lacks distance"))?;
    let times = payload
        .get("time")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("matrix payload lacks time"))?;
    if distances.len() != total_location_count || times.len() != total_location_count {
        return Err(invalid_response(format!(
            "matrix has {}x{} rows but the request needs {total_location_count}x{total_location_count}",
            distances.len(),
            times.len()
        )));
    }

    let mut entries = Vec::with_capacity(origin_count * destination_count);
    for (origin_index, distance_row) in distances.iter().enumerate().take(origin_count) {
        let distance_row = distance_row
            .as_array()
            .ok_or_else(|| invalid_response("matrix distance row is not a nested array"))?;
        let time_row = times
            .get(origin_index)
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_response("matrix time row is not a nested array"))?;
        for destination_index in 0..destination_count {
            let global_destination_index = origin_count + destination_index;
            let Some(distance_kilometers) = distance_row
                .get(global_destination_index)
                .and_then(Value::as_f64)
            else {
                continue;
            };
            let Some(duration_seconds) = time_row
                .get(global_destination_index)
                .and_then(Value::as_f64)
            else {
                continue;
            };
            entries.push(DistanceMatrixEntry {
                origin_index,
                destination_index,
                distance_meters: Some(distance_kilometers * METERS_PER_KILOMETER),
                duration_seconds: Some(duration_seconds),
            });
        }
    }
    Ok(DistanceMatrixResponse { entries })
}

#[cfg(test)]
mod tests {
    use super::parse_matrix_payload;
    use serde_json::{json, Value};

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    #[test]
    fn a_recorded_three_location_matrix_projects_the_wanted_cells() {
        let response =
            parse_matrix_payload(&fixture("routematrix_three_locations_unit_k.json"), 1, 2)
                .expect("the recorded matrix must project");
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

    #[test]
    fn two_origins_select_the_destination_column_from_the_union() {
        let response =
            parse_matrix_payload(&fixture("routematrix_three_locations_unit_k.json"), 2, 1)
                .expect("the recorded matrix must project");
        assert_eq!(response.entries.len(), 2);
        let first = &response.entries[0];
        assert_eq!(first.origin_index, 0);
        assert_eq!(first.destination_index, 0);
        assert!((first.distance_meters.unwrap() - 2600.0).abs() < 1.0);
        assert!((first.duration_seconds.unwrap() - 320.0).abs() < 1.0);
        let second = &response.entries[1];
        assert_eq!(second.origin_index, 1);
        assert!((second.distance_meters.unwrap() - 4500.0).abs() < 1.0);
        assert!((second.duration_seconds.unwrap() - 514.0).abs() < 1.0);
    }

    #[test]
    fn null_cells_are_omitted() {
        let payload = json!({
            "distance": [[0, 1.5, 2.5], [6.0, 7.0, 8.0], [9.0, 10.0, 11.0]],
            "time": [[5, null, 20], [30, 40, 50], [60, 70, 80]]
        });
        let response = parse_matrix_payload(&payload, 1, 2)
            .expect("a null cell is an unroutable pair, not a failure");
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].destination_index, 1);
        assert!((response.entries[0].distance_meters.unwrap() - 2500.0).abs() < 1.0);
    }

    #[test]
    fn a_row_count_mismatch_is_rejected() {
        let payload = json!({"distance": [[0, 1]], "time": [[0, 1], [0, 1]]});
        assert!(parse_matrix_payload(&payload, 1, 2).is_err());
    }
}
