pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::matching::{
    MatchedPoint, MatchingOptions, RouteMatcher, TraceResponse,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const ROADS_BASE_URL: &str = "https://roads.googleapis.com/v1";

/// Implementation of RouteMatcher for Google Maps Roads API.
///
/// Uses the `snapToRoads` endpoint to snap GPS points to the road network.
pub struct GoogleRouteMatcher {
    pub(crate) client: Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleRouteMatcher {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: ROADS_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core `MatchingOptions` to Google-specific parameters.
fn matching_options_from_core(options: &MatchingOptions) -> GoogleMatchOptions {
    let mut google_opts = GoogleMatchOptions::default();

    if options.transport_mode.is_some() {
        log::warn!("Google Roads API snapToRoads does not support transport_mode; ignoring");
    }
    if options.heading.is_some() {
        log::warn!("Google Roads API snapToRoads does not support heading; ignoring");
    }
    if options.departure_time.is_some() {
        log::warn!("Google Roads API snapToRoads does not support departure_time; ignoring");
    }
    if !options.avoid.is_empty() {
        log::warn!("Google Roads API snapToRoads does not support avoid restrictions; ignoring");
    }

    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("interpolate").and_then(|v| v.as_bool()) {
                google_opts.interpolate = v;
            }
            if let Some(v) = obj.get("snapping").and_then(|v| v.as_str()) {
                google_opts.snapping = Some(v.to_string());
            }
        }
    }

    google_opts
}

#[async_trait]
impl RouteMatcher for GoogleRouteMatcher {
    async fn match_route(
        &self,
        points: &[Coordinate],
        options: &MatchingOptions,
    ) -> EveryMapResult<TraceResponse> {
        let google_opts = matching_options_from_core(options);

        // Google snapToRoads requires at least 2 points, max 100
        let path: String = points
            .iter()
            .map(|p| format!("{},{}", p.lat, p.lng))
            .collect::<Vec<_>>()
            .join("|");

        let mut params: Vec<(&str, String)> = vec![("path", path)];

        if google_opts.interpolate {
            params.push(("interpolate", "true".to_string()));
        }

        let url = format!("{}/snapToRoads", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response: GoogleSnapResponse = self.client.request_json(builder).await?;

        let matched_points: Vec<MatchedPoint> = response
            .snapped_points
            .into_iter()
            .map(|sp| {
                MatchedPoint {
                    coordinate: Coordinate::new(sp.location.latitude, sp.location.longitude)
                        .unwrap_or(Coordinate::ORIGIN),
                    confidence: None, // Google doesn't provide confidence scores
                    road_name: sp.place_id, // Use place_id as an identifier
                }
            })
            .collect();

        // Google doesn't provide distance/duration in snapToRoads response
        Ok(TraceResponse {
            matched_points,
            distance: 0.0,
            duration: None,
            raw: None,
        })
    }
}
