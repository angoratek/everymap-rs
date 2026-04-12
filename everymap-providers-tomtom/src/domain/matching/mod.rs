pub mod types;

use async_trait::async_trait;
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions, TraceResponse, MatchedPoint};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::TomTomClient;
use std::sync::Arc;

pub use types::*;

const ROADS_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of RouteMatcher for TomTom Snap to Roads API.
pub struct TomTomRouteMatcher {
    client: Arc<TomTomClient>,
    base_url: String,
}

impl TomTomRouteMatcher {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: ROADS_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl RouteMatcher for TomTomRouteMatcher {
    async fn match_route(&self, points: &[Coordinate], options: &MatchingOptions) -> EveryMapResult<TraceResponse> {
        let url = format!("{}/snapToRoads/1", self.base_url);

        // TomTom uses comma-separated lat,lon pairs separated by colons
        let points_str: String = points.iter()
            .map(|p| format!("{},{}", p.lat, p.lng))
            .collect::<Vec<_>>()
            .join(":");

        let mut params: Vec<(&str, String)> = vec![("points", points_str)];

        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("fields").and_then(|v| v.as_str()) {
                    params.push(("fields", v.to_string()));
                }
            }
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: TomTomSnapResponse = self.client.request_json(builder).await?;

        let matched_points: Vec<MatchedPoint> = result.snapped_points.into_iter().map(|sp| {
            MatchedPoint {
                coordinate: sp.coordinate.map(|c| Coordinate::new(c.latitude, c.longitude).unwrap_or_else(|_| Coordinate::new(0.0, 0.0).unwrap()))
                    .unwrap_or_else(|| Coordinate::new(0.0, 0.0).unwrap()),
                confidence: None,
                road_name: None,
            }
        }).collect();

        Ok(TraceResponse {
            matched_points,
            distance: 0.0,
            duration: None,
            raw: None,
        })
    }
}