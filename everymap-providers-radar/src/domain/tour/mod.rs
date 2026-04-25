pub mod types;

use crate::client::RadarClient;
use async_trait::async_trait;
use everymap_core::domains::tour::{TourOptions, TourResponse, TourStop};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
pub use types::*;

const OPTIMIZE_BASE_URL: &str = "https://api.radar.io/v1/route/optimize";

/// Implementation of `TourPlanner` for Radar Optimize Route API.
pub struct RadarTourPlanner {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
}

impl RadarTourPlanner {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: OPTIMIZE_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl everymap_core::domains::tour::TourPlanner for RadarTourPlanner {
    async fn optimize_tour(
        &self,
        stops: &[Coordinate],
        _options: &TourOptions,
    ) -> EveryMapResult<TourResponse> {
        if stops.len() < 2 {
            return Err(EveryMapError::ValidationError(
                "At least 2 stops are required for tour optimization".to_string(),
            ));
        }

        let locations: Vec<String> = stops
            .iter()
            .map(|s| format!("{},{}", s.lat, s.lng))
            .collect();

        let mut params: Vec<(&str, String)> = vec![
            ("locations", locations.join("|")),
            ("geometry", "polyline6".to_string()),
        ];

        // Extract Radar-specific options
        if let Some(extra) = &_options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("mode").and_then(|v| v.as_str()) {
                    params.push(("mode", v.to_string()));
                }
                if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                    params.push(("units", v.to_string()));
                }
                if let Some(v) = obj.get("departureTime").and_then(|v| v.as_str()) {
                    params.push(("departureTime", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.base_url)
            .query(&params);

        let radar_res: RadarOptimizeResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!(
                    "Optimize request failed with status {}",
                    radar_res.meta.code
                ),
            ));
        }

        let tour_stops: Vec<TourStop> = radar_res
            .route
            .legs
            .iter()
            .flat_map(|leg| {
                let start = TourStop {
                    coordinate: Coordinate::new(
                        leg.start_location.latitude,
                        leg.start_location.longitude,
                    )
                    .unwrap_or(Coordinate::ORIGIN),
                    arrival_time: None,
                    departure_time: None,
                    duration: Some(leg.duration.value * 60.0),
                    distance_from_previous: Some(leg.distance.value),
                };
                vec![start]
            })
            .collect();

        // Add the final destination
        let final_stop = radar_res.route.legs.last().map(|leg| TourStop {
            coordinate: Coordinate::new(leg.end_location.latitude, leg.end_location.longitude)
                .unwrap_or(Coordinate::ORIGIN),
            arrival_time: None,
            departure_time: None,
            duration: None,
            distance_from_previous: None,
        });

        let mut all_stops = tour_stops;
        if let Some(fs) = final_stop {
            all_stops.push(fs);
        }

        Ok(TourResponse {
            stops: all_stops,
            total_distance: Some(radar_res.route.distance.value),
            total_duration: Some(radar_res.route.duration.value * 60.0),
            unassigned_count: None,
            raw: Some(serde_json::to_value(radar_res).unwrap_or_default()),
        })
    }
}
