pub mod types;

use crate::client::TomTomClient;
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    AvoidType, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult,
    RouteStrategy, Router, TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Coordinate, Polyline};
use std::sync::Arc;

pub use types::*;

const ROUTING_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of Router for TomTom Routing API.
pub struct TomTomRouter {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomRouter {
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

/// Convert core TransportMode to TomTom travel mode string.
fn transport_mode_to_tomtom(mode: &TransportMode) -> &'static str {
    match mode {
        TransportMode::Car => "car",
        TransportMode::Truck => "truck",
        TransportMode::Pedestrian => "pedestrian",
        TransportMode::Bicycle => "bicycle",
        TransportMode::Bus => "bus",
        unsupported => {
            log::warn!(
                "TomTom routing does not support transport mode {:?}, falling back to car",
                unsupported
            );
            "car"
        }
    }
}

impl TryFrom<TomTomRoute> for RouteResult {
    type Error = EveryMapError;

    fn try_from(route: TomTomRoute) -> Result<Self, Self::Error> {
        let summary = route
            .summary
            .ok_or_else(|| invalid_response("route lacks a summary"))?;
        if route.legs.is_empty() {
            return Err(invalid_response("route carries no legs"));
        }
        let legs: Vec<RouteLeg> = route
            .legs
            .into_iter()
            .map(RouteLeg::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;
        let points: Vec<Coordinate> = legs
            .iter()
            .flat_map(|leg| leg.shape.iter().copied())
            .collect();
        if points.is_empty() {
            return Err(invalid_response("route carries no geometry"));
        }

        let distance = summary
            .length_in_meters
            .ok_or_else(|| invalid_response("route summary lacks a length"))?;
        let duration = summary
            .travel_time_in_seconds
            .ok_or_else(|| invalid_response("route summary lacks a travel time"))?;

        Ok(Self {
            distance: distance as f64,
            duration: duration as f64,
            geometry: Polyline::new(points),
            transport_mode: None,
            // The basic response carries no step-level instructions.
            steps: Vec::new(),
            bounding_box: None,
            raw: None,
            legs,
        })
    }
}

impl TryFrom<TomTomRouteLeg> for RouteLeg {
    type Error = EveryMapError;

    fn try_from(leg: TomTomRouteLeg) -> Result<Self, Self::Error> {
        let summary = leg
            .summary
            .ok_or_else(|| invalid_response("route leg lacks a summary"))?;
        let shape = leg
            .points
            .into_iter()
            .map(|point| {
                let latitude = point
                    .latitude
                    .ok_or_else(|| invalid_response("route leg point lacks a latitude"))?;
                let longitude = point
                    .longitude
                    .ok_or_else(|| invalid_response("route leg point lacks a longitude"))?;
                Coordinate::new(latitude, longitude)
                    .map_err(|_| invalid_response("route leg carries an invalid point"))
            })
            .collect::<EveryMapResult<Vec<_>>>()?;
        let distance = summary
            .length_in_meters
            .ok_or_else(|| invalid_response("route leg summary lacks a length"))?;
        let duration = summary
            .travel_time_in_seconds
            .ok_or_else(|| invalid_response("route leg summary lacks a travel time"))?;
        Ok(Self {
            distance: distance as f64,
            duration: duration as f64,
            steps: Vec::new(),
            shape,
        })
    }
}

#[async_trait]
impl Router for TomTomRouter {
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        let mode = options
            .transport_mode
            .as_ref()
            .map(|m| transport_mode_to_tomtom(m))
            .unwrap_or("car");
        let url = format!(
            "{}/routing/1/calculateRoute/{},{}:{},{}/json",
            self.base_url, start.lat, start.lng, end.lat, end.lng
        );

        let mut params: Vec<(&str, String)> = vec![("travelMode", mode.to_string())];

        if let Some(alternatives) = options.alternatives {
            params.push(("maxAlternatives", alternatives.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        for avoid_type in &options.avoid {
            match avoid_type {
                AvoidType::Tolls => params.push(("avoid", "tollRoads".to_string())),
                AvoidType::Ferries => params.push(("avoid", "ferries".to_string())),
                AvoidType::Tunnels => params.push(("avoid", "tunnels".to_string())),
                AvoidType::Highways => params.push(("avoid", "motorways".to_string())),
                AvoidType::DirtRoads => params.push(("avoid", "unpavedRoads".to_string())),
            }
        }
        if let Some(departure) = &options.departure_time {
            params.push(("departAt", departure.to_string()));
        }
        if let Some(arrival) = &options.arrival_time {
            params.push(("arriveAt", arrival.to_string()));
        }
        let mut provider_route_type = None;
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("vehicleMaxSpeed").and_then(|v| v.as_u64()) {
                    params.push(("vehicleMaxSpeed", v.to_string()));
                }
                if let Some(v) = obj.get("hilliness").and_then(|v| v.as_str()) {
                    params.push(("hilliness", v.to_string()));
                }
                if let Some(v) = obj.get("routeType").and_then(|v| v.as_str()) {
                    provider_route_type = Some(v.to_string());
                    params.push(("routeType", v.to_string()));
                }
                if let Some(v) = obj.get("traffic").and_then(|v| v.as_bool()) {
                    params.push(("traffic", v.to_string()));
                }
            }
        }
        // The contract's strategy is honoured through the API's route type; a
        // provider-extra route type stays authoritative for power users.
        if provider_route_type.is_none() && options.route_strategy == Some(RouteStrategy::Shortest)
        {
            params.push(("routeType", "shortest".to_string()));
        }

        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let result: TomTomRouteResponse = self.client.request_parsed_json(builder).await?;
        let routes = result
            .routes
            .into_iter()
            .map(RouteResult::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;
        Ok(RouteResponse { routes })
    }

    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Truck,
                TransportMode::Pedestrian,
                TransportMode::Bicycle,
                TransportMode::Bus,
            ],
            // The Routing API selects a distance-minimizing path through
            // `routeType=shortest`.
            supports_shortest_route_strategy: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_route_without_a_summary_is_invalid() {
        let route: TomTomRoute =
            serde_json::from_value(json!({"legs": []})).expect("the fixture deserializes");
        match RouteResult::try_from(route) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn route_legs_are_populated_from_the_response() {
        let route: TomTomRoute = serde_json::from_value(json!({
            "summary": { "lengthInMeters": 1000, "travelTimeInSeconds": 60 },
            "legs": [{
                "summary": { "lengthInMeters": 1000, "travelTimeInSeconds": 60 },
                "points": [
                    { "latitude": 52.52, "longitude": 13.405 },
                    { "latitude": 51.5, "longitude": -0.12 }
                ]
            }]
        }))
        .expect("the fixture deserializes");
        let result = RouteResult::try_from(route).expect("the fixture converts");
        assert_eq!(result.legs.len(), 1);
        assert_eq!(result.legs[0].distance, 1000.0);
        assert_eq!(result.geometry.points.len(), 2);
    }
}
