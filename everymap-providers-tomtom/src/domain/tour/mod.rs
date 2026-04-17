pub mod types;

use async_trait::async_trait;
use everymap_core::domains::tour::{TourPlanner, TourOptions, TourResponse, TourStop};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::TomTomClient;
use std::sync::Arc;

pub use types::*;

const ROUTING_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of TourPlanner for TomTom Waypoint Optimization API.
pub struct TomTomTourPlanner {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomTourPlanner {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: ROUTING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl TourPlanner for TomTomTourPlanner {
    async fn optimize_tour(&self, stops: &[Coordinate], _options: &TourOptions) -> EveryMapResult<TourResponse> {
        if stops.len() < 2 {
            return Err(everymap_core::error::EveryMapError::provider("tomtom", "INVALID_INPUT", "At least 2 stops required for tour optimization"));
        }

        let url = format!("{}/routing/waypointoptimization/1", self.base_url);

        // Build waypoints JSON body
        let waypoints: Vec<serde_json::Value> = stops.iter()
            .map(|c| serde_json::json!({
                "point": { "latitude": c.lat, "longitude": c.lng }
            }))
            .collect();

        let body = serde_json::json!({
            "waypoints": waypoints
        });

        let builder = self.client.build_request(reqwest::Method::POST, &url)
            .json(&body);

        let result: TomTomOptimizationResponse = self.client.request_json(builder).await?;

        // Map optimized order back to stops in optimized sequence
        let tour_stops: Vec<TourStop> = result.optimized_order.into_iter()
            .filter_map(|idx| {
                let i = idx as usize;
                stops.get(i).map(|c| TourStop {
                    coordinate: *c,
                    arrival_time: None,
                    departure_time: None,
                    duration: None,
                    distance_from_previous: None,
                })
            })
            .collect();

        let (total_distance, total_duration) = result.summary.and_then(|s| {
            s.route_summary.map(|rs| {
                (rs.length_in_meters.unwrap_or(0.0), rs.travel_time_in_seconds.unwrap_or(0.0))
            })
        }).unwrap_or((0.0, 0.0));

        Ok(TourResponse {
            stops: tour_stops,
            total_distance: Some(total_distance),
            total_duration: Some(total_duration),
            unassigned_count: Some(0),
            raw: None,
        })
    }
}