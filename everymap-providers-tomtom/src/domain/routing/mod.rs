pub mod types;

use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    RouteOptions, RouteResponse, RouteResult, Router, TransportMode,
};
use everymap_core::error::EveryMapResult;
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
        TransportMode::Scooter => "car", // TomTom doesn't have scooter mode
        TransportMode::Bus => "bus",
        TransportMode::Taxi => "car",
        TransportMode::Unknown => "car",
    }
}

impl From<TomTomRoute> for RouteResult {
    fn from(route: TomTomRoute) -> Self {
        let summary = route.summary.unwrap_or_default();
        let points: Vec<Coordinate> = route
            .legs
            .iter()
            .flat_map(|leg| leg.points.iter())
            .map(|p| Coordinate::new(p.latitude, p.longitude).unwrap_or(Coordinate::ORIGIN))
            .collect();

        RouteResult {
            distance: summary.length_in_meters as f64,
            duration: summary.travel_time_in_seconds as f64,
            geometry: Polyline::new(points),
            transport_mode: None,
            steps: Vec::new(), // TomTom doesn't provide step-level instructions in basic response
            bounding_box: None,
            raw: None,
        }
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
        if let Some(departure) = &options.departure_time {
            params.push(("departAt", departure.to_string()));
        }
        if let Some(arrival) = &options.arrival_time {
            params.push(("arriveAt", arrival.to_string()));
        }
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("vehicleMaxSpeed").and_then(|v| v.as_u64()) {
                    params.push(("vehicleMaxSpeed", v.to_string()));
                }
                if let Some(v) = obj.get("hilliness").and_then(|v| v.as_str()) {
                    params.push(("hilliness", v.to_string()));
                }
                if let Some(v) = obj.get("routeType").and_then(|v| v.as_str()) {
                    params.push(("routeType", v.to_string()));
                }
                if let Some(v) = obj.get("traffic").and_then(|v| v.as_bool()) {
                    params.push(("traffic", v.to_string()));
                }
            }
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: TomTomRouteResponse = self.client.request_json(builder).await?;
        Ok(RouteResponse {
            routes: result.routes.into_iter().map(|r| r.into()).collect(),
        })
    }
}
