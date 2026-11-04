//! Settled place search through the MapQuest Search Ahead API v3 prediction
//! endpoint.
//!
//! A settled search addresses the same endpoint as type-ahead completion but
//! asks for the documented maximum of fifteen candidates. The prediction
//! payload's `results[]` become [`SearchResult`] values; the feature's
//! collection class chooses the granularity, and a result with no point is
//! skipped because a settled result must be placeable on a map.

use crate::autocomplete::{
    bounding_box_center, parse_prediction_payload, COLLECTION_PARAMETER, LIMIT_PARAMETER,
    LOCATION_PARAMETER, PREDICTION_PATH, QUERY_COLLECTIONS, QUERY_PARAMETER,
};
use crate::client::MapQuestClient;
use async_trait::async_trait;
use everymap_core::domains::search::{
    PlaceSearchOptions, PlaceSearchProvider, SearchQuality, SearchResponse, SearchResult,
    SearchResultType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Address;
use std::sync::Arc;

/// Documented maximum number of settled results, used when the caller sets no
/// cap.
const DEFAULT_SETTLED_SEARCH_LIMIT: u32 = 15;

/// Collection values describing an address-level match.
const ADDRESS_COLLECTIONS: [&str; 1] = ["address"];

/// Collection values describing a point of interest.
const POINT_COLLECTIONS: [&str; 4] = ["poi", "category", "franchise", "airport"];

/// Collection values describing an administrative area.
const AREA_COLLECTIONS: [&str; 1] = ["adminArea"];

/// Settled place search over the MapQuest Search Ahead prediction endpoint.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`].
pub struct MapQuestPlaceSearchProvider {
    client: Arc<MapQuestClient>,
}

impl MapQuestPlaceSearchProvider {
    /// Builds a settled search provider around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A provider ready to answer settled queries.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PlaceSearchProvider for MapQuestPlaceSearchProvider {
    async fn search_places(
        &self,
        query: &str,
        options: &PlaceSearchOptions,
    ) -> EveryMapResult<SearchResponse> {
        if query.trim().is_empty() {
            return Err(EveryMapError::ValidationError(
                "search query text must not be empty".to_string(),
            ));
        }
        let result_limit = options.limit.unwrap_or(DEFAULT_SETTLED_SEARCH_LIMIT);
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
        if options.provider_extra.is_some() {
            log::warn!("MapQuest Search Ahead does not consume provider_extra; ignoring");
        }
        let payload = self
            .client
            .get_json(PREDICTION_PATH, &query_parameters)
            .await?;
        let candidates = parse_prediction_payload(&payload)?;
        Ok(SearchResponse {
            items: candidates
                .items
                .into_iter()
                .filter_map(settled_result)
                .collect(),
        })
    }
}

/// Projects one completion candidate into a settled search result, or `None`
/// when the candidate has no position.
fn settled_result(
    candidate: everymap_core::domains::autocomplete::AutocompleteResult,
) -> Option<SearchResult> {
    let coordinate = candidate.coordinate?;
    let quality = quality_for_collections(&candidate.categories);
    Some(SearchResult {
        id: candidate.id,
        coordinate,
        address: Address {
            label: Some(candidate.title.clone()),
            ..Address::empty()
        },
        title: Some(candidate.title),
        result_type: SearchResultType::ExactMatch,
        quality,
        distance: None,
        confidence: None,
        categories: candidate.categories,
        bounding_box: candidate.bounding_box,
        raw: candidate.raw,
    })
}

/// Chooses the granularity a prediction candidate's collection reports.
fn quality_for_collections(collections: &[String]) -> Option<SearchQuality> {
    if collections
        .iter()
        .any(|name| ADDRESS_COLLECTIONS.contains(&name.as_str()))
    {
        return Some(SearchQuality::StreetAddress);
    }
    if collections
        .iter()
        .any(|name| POINT_COLLECTIONS.contains(&name.as_str()))
    {
        return Some(SearchQuality::Point);
    }
    if collections
        .iter()
        .any(|name| AREA_COLLECTIONS.contains(&name.as_str()))
    {
        return Some(SearchQuality::City);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_and_point_collections_choose_their_granularity() {
        assert_eq!(
            quality_for_collections(&["address".to_string()]),
            Some(SearchQuality::StreetAddress)
        );
        assert_eq!(
            quality_for_collections(&["poi".to_string()]),
            Some(SearchQuality::Point)
        );
        assert_eq!(
            quality_for_collections(&["adminArea".to_string()]),
            Some(SearchQuality::City)
        );
        assert_eq!(quality_for_collections(&[]), None);
    }
}
