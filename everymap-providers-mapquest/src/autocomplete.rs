//! Type-ahead completion through the MapQuest Search Ahead API v3 prediction
//! endpoint.
//!
//! One request is `GET {base_url}/search/v3/prediction` with the partial query
//! as `q`, the documented collection list, the result cap and, when the caller
//! supplies a preferred area, a ranking centre derived from that area's
//! midpoint as `location={longitude},{latitude}`.
//!
//! The endpoint returns GeoJSON-like `results[]` features. Each feature's
//! `displayString` becomes the suggestion's primary line, its
//! `place.geometry.coordinates` becomes the suggested point, and its
//! `collection` values become the suggestion's categories.

use crate::client::MapQuestClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::autocomplete::{
    AutocompleteOptions, AutocompleteProvider, AutocompleteResponse, AutocompleteResult,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{BoundingBox, Coordinate};
use serde_json::Value;
use std::sync::Arc;

/// Service path of the prediction endpoint.
pub(crate) const PREDICTION_PATH: &str = "search/v3/prediction";

/// Collections queried on every prediction call.
pub(crate) const QUERY_COLLECTIONS: &str = "address,adminArea,airport,category,franchise,poi";

/// Result cap used when the caller does not set one.
const DEFAULT_AUTOCOMPLETE_LIMIT: u32 = 5;

/// Query parameter carrying the partial query text.
pub(crate) const QUERY_PARAMETER: &str = "q";

/// Query parameter naming the collections to search.
pub(crate) const COLLECTION_PARAMETER: &str = "collection";

/// Query parameter capping the number of candidates.
pub(crate) const LIMIT_PARAMETER: &str = "limit";

/// Query parameter naming the ranking centre.
pub(crate) const LOCATION_PARAMETER: &str = "location";

/// Type-ahead completion through the MapQuest Search Ahead prediction
/// endpoint.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`].
pub struct MapQuestAutocompleteProvider {
    client: Arc<MapQuestClient>,
}

impl MapQuestAutocompleteProvider {
    /// Builds an autocomplete provider around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A provider ready to complete partial queries.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AutocompleteProvider for MapQuestAutocompleteProvider {
    /// Returns completion candidates for a partial query.
    ///
    /// # Parameters
    /// - `query` — the text entered so far. A blank value is rejected before
    ///   any request is sent.
    /// - `options` — the result cap and a preferred area. The area is passed
    ///   as the documented single ranking centre at its midpoint; the endpoint
    ///   has no language, country-restriction or provider-extra parameter in
    ///   this request, so those options are reported as unsupported through
    ///   the log.
    ///
    /// # Returns
    /// Candidates in the service's relevance order, at most the requested
    /// cap.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when `query` is blank.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when the body is not JSON.
    /// - [`EveryMapError::ProviderError`] when a result lacks its display
    ///   string or geometry.
    async fn autocomplete(
        &self,
        query: &str,
        options: &AutocompleteOptions,
    ) -> EveryMapResult<AutocompleteResponse> {
        if query.trim().is_empty() {
            return Err(EveryMapError::ValidationError(
                "search query text must not be empty".to_string(),
            ));
        }
        let result_limit = options.limit.unwrap_or(DEFAULT_AUTOCOMPLETE_LIMIT);
        let mut query_parameters = vec![
            (QUERY_PARAMETER.to_string(), query.to_string()),
            (
                COLLECTION_PARAMETER.to_string(),
                QUERY_COLLECTIONS.to_string(),
            ),
            (LIMIT_PARAMETER.to_string(), result_limit.to_string()),
        ];
        if let Some(bounding_box) = options.bounding_box {
            let ranking_center = bounding_box_center(&bounding_box)?;
            // Documented format: longitude, latitude.
            query_parameters.push((
                LOCATION_PARAMETER.to_string(),
                format!("{},{}", ranking_center.lng, ranking_center.lat),
            ));
        }
        if options.language.is_some() {
            log::warn!(
                "MapQuest Search Ahead does not support a language parameter in this request; ignoring"
            );
        }
        if !options.country_codes.is_empty() {
            log::warn!(
                "MapQuest Search Ahead does not support country restriction in this request; ignoring"
            );
        }
        if options.provider_extra.is_some() {
            log::warn!("MapQuest Search Ahead does not consume provider_extra; ignoring");
        }
        let payload = self
            .client
            .get_json(PREDICTION_PATH, &query_parameters)
            .await?;
        parse_prediction_payload(&payload)
    }
}

/// Returns the midpoint of a bounding box.
pub(crate) fn bounding_box_center(bounding_box: &BoundingBox) -> EveryMapResult<Coordinate> {
    Coordinate::new(
        (bounding_box.south_west.lat + bounding_box.north_east.lat) / 2.0,
        (bounding_box.south_west.lng + bounding_box.north_east.lng) / 2.0,
    )
    .map_err(|error| EveryMapError::ValidationError(error.to_string()))
}

/// Projects a prediction payload into vendor-neutral completion candidates.
///
/// # Parameters
/// - `payload` — a decoded prediction response body carrying a `results`
///   array.
///
/// # Returns
/// One candidate per result, in service order.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] when `results` is missing, a
/// result lacks `displayString`, or its geometry does not carry a longitude
/// and latitude pair.
pub fn parse_prediction_payload(payload: &Value) -> EveryMapResult<AutocompleteResponse> {
    let results = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("prediction payload lacks results"))?;
    let mut items = Vec::with_capacity(results.len());
    for result in results {
        let display_string = result
            .get("displayString")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_response("prediction result lacks displayString"))?;
        // The service returns collection entries without a place, for example
        // a franchise that names a brand rather than a position. A candidate
        // with no position cannot be drawn, so it is left to the settled
        // search instead of failing the whole response.
        let Some(longitude) = result
            .pointer("/place/geometry/coordinates/0")
            .and_then(Value::as_f64)
        else {
            continue;
        };
        let Some(latitude) = result
            .pointer("/place/geometry/coordinates/1")
            .and_then(Value::as_f64)
        else {
            continue;
        };
        let coordinate = Coordinate::new(latitude, longitude)
            .map_err(|error| invalid_response(error.to_string()))?;
        let mut categories = result
            .get("collection")
            .and_then(Value::as_array)
            .map(|collections| {
                collections
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        if categories.is_empty() {
            if let Some(record_type) = result.get("recordType").and_then(Value::as_str) {
                categories.push(record_type.to_string());
            }
        }
        items.push(AutocompleteResult {
            id: result.get("id").and_then(Value::as_str).map(str::to_string),
            title: display_string.to_string(),
            subtitle: None,
            coordinate: Some(coordinate),
            bounding_box: None,
            quality: None,
            categories,
            raw: Some(result.clone()),
        });
    }
    Ok(AutocompleteResponse { items })
}

#[cfg(test)]
mod tests {
    use super::parse_prediction_payload;
    use serde_json::Value;

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    #[test]
    fn a_candidate_without_a_place_is_skipped_not_refused() {
        // The service answers a franchise entry that names a brand and carries
        // no place; it must not fail the candidates beside it.
        let payload: Value = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "collection": ["adminArea"],
                        "displayString": "Paris, Ile-de-France, France",
                        "id": "mqId:282366836",
                        "name": "Paris",
                        "place": {
                            "geometry": { "coordinates": [2.3414, 48.85717], "type": "Point" },
                            "properties": { "city": "Paris" },
                            "type": "Feature"
                        },
                        "recordType": "city"
                    },
                    {
                        "collection": ["franchise"],
                        "displayString": "Paris",
                        "id": "category:2ed9a84fb4eb19a766d2a4df4f7357dc",
                        "name": "Paris",
                        "recordType": "franchise"
                    }
                ]
            }"#,
        )
        .expect("the payload must be valid JSON");
        let response = parse_prediction_payload(&payload).expect("the payload must project");
        assert_eq!(
            response.items.len(),
            1,
            "the franchise entry has no position and is skipped"
        );
        assert_eq!(response.items[0].title, "Paris, Ile-de-France, France");
    }

    #[test]
    fn a_recorded_prediction_projects_every_candidate() {
        let response = parse_prediction_payload(&fixture("search_prediction_denver.json"))
            .expect("the recorded payload must project");
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

    #[test]
    fn a_result_without_a_display_string_is_rejected() {
        let payload = serde_json::json!({
            "results": [{
                "place": {"geometry": {"coordinates": [-104.0, 39.0]}}
            }]
        });
        assert!(parse_prediction_payload(&payload).is_err());
    }
}
