pub mod types;

use crate::client::MapBoxClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult, RouteStep, Router,
    TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
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
        TransportMode::Truck => {
            log::warn!(
                "MapBox Directions API does not support truck profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Pedestrian => "walking",
        TransportMode::Bicycle => "cycling",
        TransportMode::Scooter => {
            log::warn!(
                "MapBox Directions API does not support scooter profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Bus => {
            log::warn!(
                "MapBox Directions API does not support bus profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Taxi => {
            log::warn!(
                "MapBox Directions API does not support taxi profile, falling back to driving"
            );
            "driving"
        }
        TransportMode::Unknown => "driving",
    }
}

/// Number of decimal places in the `polyline` geometry the Directions API
/// returns for the `geometries=polyline` request.
const MAPBOX_POLYLINE_PRECISION: u8 = 5;

/// The failure a route carries when its encoded shape cannot be decoded.
fn invalid_polyline(detail: String) -> EveryMapError {
    EveryMapError::provider("mapbox", "INVALID_POLYLINE", detail)
}

impl TryFrom<MapBoxRoute> for RouteResult {
    type Error = EveryMapError;

    fn try_from(route: MapBoxRoute) -> Result<Self, Self::Error> {
        let distance = route
            .distance
            .ok_or_else(|| invalid_response("route lacks a distance"))?;
        let duration = route
            .duration
            .ok_or_else(|| invalid_response("route lacks a duration"))?;

        // The Directions API answers `geometries=polyline` with the classic
        // Google encoding, not HERE's flexible polyline; decoding it with the
        // wrong codec would silently report an empty shape.
        let encoded = route
            .geometry
            .as_deref()
            .filter(|geometry| !geometry.is_empty())
            .ok_or_else(|| invalid_response("route lacks an encoded polyline"))?;
        let points =
            everymap_core::types::decode_encoded_polyline(encoded, MAPBOX_POLYLINE_PRECISION)
                .map_err(invalid_polyline)?;

        let legs: Vec<RouteLeg> = route
            .legs
            .into_iter()
            .map(RouteLeg::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;
        let steps: Vec<RouteStep> = legs
            .iter()
            .flat_map(|leg| leg.steps.iter().cloned())
            .collect();

        Ok(RouteResult {
            distance,
            duration,
            geometry: Polyline::new(points),
            transport_mode: None,
            steps,
            bounding_box: None,
            raw: None,
            legs,
        })
    }
}

impl TryFrom<MapBoxRouteLeg> for RouteLeg {
    type Error = EveryMapError;

    fn try_from(leg: MapBoxRouteLeg) -> Result<Self, Self::Error> {
        let distance = leg
            .distance
            .ok_or_else(|| invalid_response("route leg lacks a distance"))?;
        let duration = leg
            .duration
            .ok_or_else(|| invalid_response("route leg lacks a duration"))?;
        let steps = leg.steps.into_iter().map(route_step_from_mapbox).collect();
        Ok(Self {
            distance,
            duration,
            steps,
            shape: Vec::new(),
        })
    }
}

/// Projects one Directions API step onto the contract's route step.
fn route_step_from_mapbox(step: MapBoxRouteStep) -> RouteStep {
    RouteStep {
        instruction: step.instruction.or(step.name),
        distance: step.distance,
        duration: step.duration,
        start_coordinate: step.maneuver.and_then(|maneuver| {
            maneuver.location.and_then(|location| {
                if location.len() >= 2 {
                    Coordinate::new(location[1], location[0]).ok()
                } else {
                    None
                }
            })
        }),
        end_coordinate: None,
        maneuver_type: None,
        speech_instruction: None,
        lane_guidances: Vec::new(),
        shape: Vec::new(),
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
        if options.departure_time.is_some() {
            log::warn!(
                "MapBox Directions API v5 does not support departure_time; \
                 departure_time will be ignored"
            );
        }
        if options.arrival_time.is_some() {
            log::warn!(
                "MapBox Directions API v5 does not support arrival_time; \
                 arrival_time will be ignored"
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: MapBoxRouteResponse = self.client.request_parsed_json(builder).await?;
        let routes = result
            .routes
            .into_iter()
            .map(RouteResult::try_from)
            .collect::<everymap_core::error::EveryMapResult<Vec<_>>>()?;
        Ok(RouteResponse { routes })
    }

    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Pedestrian,
                TransportMode::Bicycle,
            ],
            // The Directions API ranks paths for travel time; it exposes no
            // distance-minimizing strategy.
            supports_shortest_route_strategy: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_route_without_a_distance_is_invalid() {
        let route: MapBoxRoute = serde_json::from_value(json!({
            "duration": 60.0,
            "geometry": "_yp_IgdypAfojU~vmbA"
        }))
        .expect("the fixture deserializes");
        match RouteResult::try_from(route) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn route_legs_are_populated_from_the_response() {
        let route: MapBoxRoute = serde_json::from_value(json!({
            "distance": 1000.0,
            "duration": 60.0,
            "geometry": "_yp_IgdypAfojU~vmbA",
            "legs": [{
                "distance": 1000.0,
                "duration": 60.0,
                "steps": [{
                    "distance": 1000.0,
                    "duration": 60.0,
                    "instruction": "Head north"
                }]
            }]
        }))
        .expect("the fixture deserializes");
        let result = RouteResult::try_from(route).expect("the fixture converts");
        assert_eq!(result.legs.len(), 1);
        assert_eq!(result.legs[0].steps.len(), 1);
        assert_eq!(result.steps.len(), 1);
        assert!(!result.geometry.points.is_empty());
    }
}
