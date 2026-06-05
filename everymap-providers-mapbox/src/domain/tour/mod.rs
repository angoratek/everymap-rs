pub mod types;

use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::tour::{TourOptions, TourPlanner, TourResponse, TourStop};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const OPTIMIZATION_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of TourPlanner for MapBox Optimization API v1.
pub struct MapBoxTourPlanner {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxTourPlanner {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: OPTIMIZATION_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl TourPlanner for MapBoxTourPlanner {
    async fn optimize_tour(
        &self,
        stops: &[Coordinate],
        options: &TourOptions,
    ) -> EveryMapResult<TourResponse> {
        if stops.len() < 2 {
            return Err(everymap_core::error::EveryMapError::provider(
                "mapbox",
                "INVALID_INPUT",
                "At least 2 stops required for tour optimization",
            ));
        }

        let profile = options
            .transport_mode
            .as_ref()
            .map(|m| match m {
                everymap_core::domains::routing::TransportMode::Car => "driving",
                everymap_core::domains::routing::TransportMode::Truck => "driving",
                everymap_core::domains::routing::TransportMode::Pedestrian => "walking",
                everymap_core::domains::routing::TransportMode::Bicycle => "cycling",
                _ => "driving",
            })
            .unwrap_or("driving");

        let coords: String = stops
            .iter()
            .map(|c| format!("{},{}", c.lng, c.lat))
            .collect::<Vec<_>>()
            .join(";");
        let url = format!(
            "{}/optimized-trips/v1/mapbox/{}/{}",
            self.base_url, profile, coords
        );

        let mut params: Vec<(&str, String)> = vec![
            ("overview", "false".to_string()),
            ("roundtrip", "false".to_string()),
        ];

        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("source").and_then(|v| v.as_str()) {
                    params.push(("source", v.to_string()));
                }
                if let Some(v) = obj.get("destination").and_then(|v| v.as_str()) {
                    params.push(("destination", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxOptimizationResponse = self.client.request_json(builder).await?;

        // Build tour stops from waypoints in optimized order
        let mut waypoints: Vec<(usize, Option<Coordinate>)> = result
            .waypoints
            .into_iter()
            .map(|wp| {
                let coordinate = wp.location.as_ref().and_then(|loc| {
                    if loc.len() >= 2 {
                        Some(Coordinate::new(loc[1], loc[0]).unwrap_or(Coordinate::ORIGIN))
                    } else {
                        None
                    }
                });
                let waypoint_index = wp.waypoint_index.unwrap_or(0) as usize;
                (waypoint_index, coordinate)
            })
            .collect();
        waypoints.sort_by_key(|(waypoint_index, _)| *waypoint_index);

        let tour_stops: Vec<TourStop> = waypoints
            .into_iter()
            .map(|(_, coordinate)| TourStop {
                coordinate: coordinate.unwrap_or(Coordinate::ORIGIN),
                arrival_time: None,
                departure_time: None,
                duration: None,
                distance_from_previous: None,
            })
            .collect();

        let (total_distance, total_duration) = result
            .trips
            .first()
            .map(|t| (t.distance, t.duration))
            .unwrap_or((0.0, 0.0));

        Ok(TourResponse {
            stops: tour_stops,
            total_distance: Some(total_distance),
            total_duration: Some(total_duration),
            unassigned_count: Some(0),
            raw: None,
        })
    }
}
