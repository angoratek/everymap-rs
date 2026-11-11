pub mod types;

use crate::client::RadarClient;
use crate::endpoints::{endpoint, OPTIMIZE_TOUR_PATH, RADAR_API_ROOT};
use crate::error_mapping::{ensure_success, invalid_response};
use async_trait::async_trait;
use everymap_core::domains::tour::{TourOptions, TourResponse, TourStop};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
pub use types::*;

/// Number of seconds in one minute, the unit Radar durations use.
const SECONDS_PER_MINUTE: f64 = 60.0;

/// Implementation of `TourPlanner` for Radar Optimize Route API.
pub struct RadarTourPlanner {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) service_root: String,
}

impl RadarTourPlanner {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            service_root: RADAR_API_ROOT.to_string(),
        }
    }

    /// Creates a tour planner against a Radar service root.
    ///
    /// # Parameters
    /// - `client` — the shared Radar client.
    /// - `service_root` — the scheme and host serving the API, for example
    ///   `https://api.radar.io` or a test server's origin. Every Radar
    ///   endpoint is derived from this one root.
    pub fn with_base_url(client: std::sync::Arc<RadarClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The tour optimization endpoint.
    fn endpoint(&self) -> String {
        endpoint(&self.service_root, OPTIMIZE_TOUR_PATH)
    }
}

#[async_trait]
impl everymap_core::domains::tour::TourPlanner for RadarTourPlanner {
    async fn optimize_tour(
        &self,
        stops: &[Coordinate],
        options: &TourOptions,
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

        // Read transport_mode from core field or provider_extra; the core
        // field maps through the same table routing and matching use, so the
        // service never receives an enum name such as "pedestrian".
        let mode = options
            .transport_mode
            .as_ref()
            .map(|m| crate::domain::transport::radar_transport_mode(m).to_string())
            .or_else(|| {
                options
                    .provider_extra
                    .as_ref()
                    .and_then(|e| e.get("mode"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        if let Some(mode_str) = mode {
            params.push(("mode", mode_str));
        }

        // Extract Radar-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                    params.push(("units", v.to_string()));
                }
                if let Some(v) = obj.get("departureTime").and_then(|v| v.as_str()) {
                    params.push(("departureTime", v.to_string()));
                }
            }
        }

        let url = self.endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let radar_res: RadarOptimizeResponse = self.client.request_parsed_json(builder).await?;
        ensure_success(radar_res.meta.code)?;

        if radar_res.route.legs.is_empty() {
            return Err(invalid_response("optimized tour carries no legs"));
        }

        let mut tour_stops: Vec<TourStop> = Vec::with_capacity(radar_res.route.legs.len() + 1);
        for leg in &radar_res.route.legs {
            let duration = leg
                .duration
                .value
                .ok_or_else(|| invalid_response("optimized leg duration lacks a value"))?
                * SECONDS_PER_MINUTE;
            let distance = leg
                .distance
                .value
                .ok_or_else(|| invalid_response("optimized leg distance lacks a value"))?;
            let start_latitude = leg
                .start_location
                .latitude
                .ok_or_else(|| invalid_response("optimized leg start lacks a latitude"))?;
            let start_longitude = leg
                .start_location
                .longitude
                .ok_or_else(|| invalid_response("optimized leg start lacks a longitude"))?;
            tour_stops.push(TourStop {
                coordinate: Coordinate::new(start_latitude, start_longitude).map_err(|_| {
                    invalid_response("optimized leg carries an invalid start location")
                })?,
                arrival_time: None,
                departure_time: None,
                duration: Some(duration),
                distance_from_previous: Some(distance),
            });
        }

        // Add the final destination.
        let last_leg = radar_res
            .route
            .legs
            .last()
            .expect("the leg list was checked to be non-empty");
        let end_latitude = last_leg
            .end_location
            .latitude
            .ok_or_else(|| invalid_response("optimized leg end lacks a latitude"))?;
        let end_longitude = last_leg
            .end_location
            .longitude
            .ok_or_else(|| invalid_response("optimized leg end lacks a longitude"))?;
        tour_stops.push(TourStop {
            coordinate: Coordinate::new(end_latitude, end_longitude)
                .map_err(|_| invalid_response("optimized leg carries an invalid end location"))?,
            arrival_time: None,
            departure_time: None,
            duration: None,
            distance_from_previous: None,
        });

        let total_distance = radar_res
            .route
            .distance
            .value
            .ok_or_else(|| invalid_response("optimized tour distance lacks a value"))?;
        let total_duration = radar_res
            .route
            .duration
            .value
            .ok_or_else(|| invalid_response("optimized tour duration lacks a value"))?
            * SECONDS_PER_MINUTE;

        Ok(TourResponse {
            stops: tour_stops,
            total_distance: Some(total_distance),
            total_duration: Some(total_duration),
            unassigned_count: None,
            raw: serde_json::to_value(&radar_res).ok(),
        })
    }
}
