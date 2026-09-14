pub mod types;

use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::matching::{
    MatchedPoint, MatchingOptions, RouteMatcher, TraceResponse,
};
use everymap_core::domains::routing::TransportMode;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const MATCHING_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of RouteMatcher for MapBox Map Matching API v5.
pub struct MapBoxRouteMatcher {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxRouteMatcher {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: MATCHING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core TransportMode to MapBox profile.
fn transport_mode_to_profile(mode: &TransportMode) -> &'static str {
    match mode {
        TransportMode::Car => "driving",
        TransportMode::Truck => {
            log::warn!(
                "MapBox Map Matching API does not support truck profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Pedestrian => "walking",
        TransportMode::Bicycle => "cycling",
        TransportMode::Bus => {
            log::warn!(
                "MapBox Map Matching API does not support bus profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Taxi => {
            log::warn!(
                "MapBox Map Matching API does not support taxi profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Scooter => {
            log::warn!(
                "MapBox Map Matching API does not support scooter profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Unknown => "driving",
    }
}

#[async_trait]
impl RouteMatcher for MapBoxRouteMatcher {
    async fn match_route(
        &self,
        points: &[Coordinate],
        options: &MatchingOptions,
    ) -> EveryMapResult<TraceResponse> {
        if points.len() < 2 {
            return Err(everymap_core::error::EveryMapError::provider(
                "mapbox",
                "INVALID_INPUT",
                "At least 2 points required for map matching",
            ));
        }

        let profile = options
            .transport_mode
            .as_ref()
            .map(|m| transport_mode_to_profile(m))
            .unwrap_or("driving");
        let coords: String = points
            .iter()
            .map(|p| format!("{},{}", p.lng, p.lat))
            .collect::<Vec<_>>()
            .join(";");
        let url = format!(
            "{}/matching/v5/mapbox/{}/{}.json",
            self.base_url, profile, coords
        );

        let mut params: Vec<(&str, String)> = vec![
            ("overview", "full".to_string()),
            ("geometries", "polyline".to_string()),
        ];

        if options.heading.is_some() {
            log::warn!("MapBox Map Matching API v5 does not support heading; ignoring");
        }
        if options.departure_time.is_some() {
            log::warn!("MapBox Map Matching API v5 does not support departure_time; ignoring");
        }
        if !options.avoid.is_empty() {
            log::warn!("MapBox Map Matching API v5 does not support avoid restrictions; ignoring");
        }

        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("tidy").and_then(|v| v.as_bool()) {
                    params.push(("tidy", v.to_string()));
                }
                if let Some(v) = obj.get("radiuses").and_then(|v| v.as_str()) {
                    params.push(("radiuses", v.to_string()));
                }
                if let Some(v) = obj.get("timestamps").and_then(|v| v.as_str()) {
                    params.push(("timestamps", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxMatchResponse = self.client.request_json(builder).await?;

        // Extract matched points from tracepoints
        let matched_points: Vec<MatchedPoint> = result
            .tracepoints
            .into_iter()
            .filter(|tp| tp.location.is_some())
            .map(MatchedPoint::from)
            .collect();

        // Get total distance/duration from the best matching
        let (distance, duration) = result
            .matchings
            .first()
            .map(|m| (m.distance, m.duration))
            .unwrap_or((0.0, 0.0));

        Ok(TraceResponse {
            matched_points,
            distance,
            duration: Some(duration),
            raw: None,
        })
    }
}
