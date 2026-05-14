pub mod types;

use crate::client::MapBoxClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    RouteOptions, RouteResponse, RouteResult, RouteStep, Router, TransportMode,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{Coordinate, Polyline};
use std::sync::Arc;

pub use types::*;

const ROUTING_BASE_URL: &str = "https://api.mapbox.com";

/// Implementation of Router for MapBox Directions API v5.
pub struct MapBoxRouter {
    pub(crate) client: Arc<MapBoxClient>,
    pub(crate) base_url: String,
}

impl MapBoxRouter {
    pub fn new(client: Arc<MapBoxClient>) -> Self {
        Self {
            client,
            base_url: ROUTING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<MapBoxClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core TransportMode to MapBox profile string.
fn transport_mode_to_profile(mode: &TransportMode) -> &'static str {
    match mode {
        TransportMode::Car => "driving",
        TransportMode::Truck => "driving",
        TransportMode::Pedestrian => "walking",
        TransportMode::Bicycle => "cycling",
        TransportMode::Scooter => "driving",
        TransportMode::Bus => "driving",
        TransportMode::Taxi => "driving",
        TransportMode::Unknown => "driving",
    }
}

impl From<MapBoxRoute> for RouteResult {
    fn from(route: MapBoxRoute) -> Self {
        // Decode polyline geometry if present; otherwise extract from legs
        let points: Vec<Coordinate> = route
            .geometry
            .as_ref()
            .and_then(|encoded| everymap_core::types::FlexiblePolyline::decode(encoded).ok())
            .unwrap_or_default();

        let steps: Vec<RouteStep> = route
            .legs
            .iter()
            .flat_map(|leg| leg.steps.iter())
            .map(|step| RouteStep {
                instruction: step.instruction.clone().or(step.name.clone()),
                distance: Some(step.distance),
                duration: Some(step.duration),
                start_coordinate: step.maneuver.as_ref().and_then(|m| {
                    m.location.as_ref().and_then(|loc| {
                        if loc.len() >= 2 {
                            Some(Coordinate::new(loc[1], loc[0]).unwrap_or(Coordinate::ORIGIN))
                        } else {
                            None
                        }
                    })
                }),
                end_coordinate: None,
            })
            .collect();

        RouteResult {
            distance: route.distance,
            duration: route.duration,
            geometry: Polyline::new(points),
            transport_mode: None,
            steps,
            bounding_box: None,
            raw: None,
        }
    }
}

#[async_trait]
impl Router for MapBoxRouter {
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        let profile = options
            .transport_mode
            .as_ref()
            .map(|m| transport_mode_to_profile(m))
            .unwrap_or("driving");
        let coords = format!("{},{};{},{}", start.lng, start.lat, end.lng, end.lat);
        let url = format!(
            "{}/directions/v5/mapbox/{}/{}",
            self.base_url, profile, coords
        );

        let mut params: Vec<(&str, String)> = vec![
            ("overview", "full".to_string()),
            ("geometries", "polyline".to_string()),
            ("steps", "true".to_string()),
        ];

        if let Some(alternatives) = options.alternatives {
            params.push(("alternatives", alternatives.to_string()));
        }
        if !options.avoid.is_empty() {
            log::warn!(
                "MapBox Directions API v5 does not support avoid restrictions; \
                 avoid will be ignored"
            );
        }
        if options.language.is_some() {
            log::warn!(
                "MapBox Directions API v5 does not support a language parameter; \
                 language will be ignored"
            );
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("annotations").and_then(|v| v.as_str()) {
                    params.push(("annotations", v.to_string()));
                }
                if let Some(v) = obj.get("continue_straight").and_then(|v| v.as_bool()) {
                    params.push(("continue_straight", v.to_string()));
                }
                if let Some(v) = obj.get("exclude").and_then(|v| v.as_str()) {
                    params.push(("exclude", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: MapBoxRouteResponse = self.client.request_json(builder).await?;
        Ok(RouteResponse {
            routes: result.routes.into_iter().map(|r| r.into()).collect(),
        })
    }
}
