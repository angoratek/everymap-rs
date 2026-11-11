pub mod types;

use crate::client::RadarClient;
use crate::endpoints::{endpoint, DIRECTIONS_PATH, DISTANCE_PATH, MATRIX_PATH, RADAR_API_ROOT};
use crate::error_mapping::{ensure_success, invalid_response};
use async_trait::async_trait;
use everymap_core::domains::routing::{
    AvoidType, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult, RouteStep,
    TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{Coordinate, Polyline};
pub use types::*;

/// Implementation of `Router` for Radar Directions API.
pub struct RadarRouter {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) service_root: String,
}

impl RadarRouter {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            service_root: RADAR_API_ROOT.to_string(),
        }
    }

    /// Creates a router against a Radar service root.
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

    /// The directions endpoint.
    fn directions_endpoint(&self) -> String {
        endpoint(&self.service_root, DIRECTIONS_PATH)
    }

    /// The distance endpoint.
    pub(crate) fn distance_endpoint(&self) -> String {
        endpoint(&self.service_root, DISTANCE_PATH)
    }

    /// The distance matrix endpoint.
    pub(crate) fn matrix_endpoint(&self) -> String {
        endpoint(&self.service_root, MATRIX_PATH)
    }
}

fn map_transport_mode(mode: &TransportMode) -> &'static str {
    match mode {
        TransportMode::Car => "car",
        TransportMode::Truck => "truck",
        TransportMode::Pedestrian | TransportMode::Scooter => "foot",
        TransportMode::Bicycle => "bike",
        unsupported => {
            log::warn!(
                "Radar Directions API does not support transport mode {:?}, falling back to car",
                unsupported
            );
            "car"
        }
    }
}

/// Map core `AvoidType` values to Radar Directions API avoid features.
/// Radar supports a comma-separated list of: tolls, highways, ferries, borderCrossings.
/// Returns the joined list, or None when nothing is supported.
fn map_avoid_types(avoid_types: &[AvoidType]) -> Option<String> {
    let mut features: Vec<&str> = Vec::new();
    for avoid_type in avoid_types {
        match avoid_type {
            AvoidType::Tolls => features.push("tolls"),
            AvoidType::Highways => features.push("highways"),
            AvoidType::Ferries => features.push("ferries"),
            unsupported => {
                log::warn!(
                    "Radar Directions API does not support avoiding {:?}; \
                     supported avoid features are tolls, highways, ferries, borderCrossings",
                    unsupported
                );
            }
        }
    }
    if features.is_empty() {
        None
    } else {
        Some(features.join(","))
    }
}

/// Number of decimal places in the `polyline6` geometry the Directions API
/// returns for the `geometry=polyline6` request.
const RADAR_POLYLINE_PRECISION: u8 = 6;

/// The failure a route carries when its encoded shape cannot be decoded.
fn invalid_polyline(detail: String) -> EveryMapError {
    EveryMapError::provider("radar", "INVALID_POLYLINE", detail)
}

/// Number of seconds in one minute, the unit Radar durations use.
const SECONDS_PER_MINUTE: f64 = 60.0;

impl TryFrom<RadarDirectionsRoute> for RouteResult {
    type Error = EveryMapError;

    fn try_from(route: RadarDirectionsRoute) -> Result<Self, Self::Error> {
        let raw = serde_json::to_value(&route).ok();
        let distance = route
            .distance
            .value
            .ok_or_else(|| invalid_response("route distance lacks a value"))?;
        let duration = route
            .duration
            .value
            .ok_or_else(|| invalid_response("route duration lacks a value"))?
            * SECONDS_PER_MINUTE;
        let geometry_str = route
            .geometry
            .as_ref()
            .and_then(|geometry| geometry.polyline.as_deref())
            .filter(|polyline| !polyline.is_empty())
            .ok_or_else(|| invalid_response("route lacks an encoded polyline"))?;
        // Radar answers `geometry=polyline6` with the Mapbox encoding; HERE's
        // flexible polyline is a different format and would decode to nothing.
        let geometry = Polyline::new(
            everymap_core::types::decode_encoded_polyline(geometry_str, RADAR_POLYLINE_PRECISION)
                .map_err(invalid_polyline)?,
        );

        let legs: Vec<RouteLeg> = route
            .legs
            .into_iter()
            .map(route_leg_from_radar)
            .collect::<EveryMapResult<Vec<_>>>()?;
        let steps: Vec<RouteStep> = legs
            .iter()
            .flat_map(|leg| leg.steps.iter().cloned())
            .collect();

        Ok(Self {
            distance,
            duration,
            geometry,
            transport_mode: None,
            steps,
            bounding_box: None,
            raw,
            legs,
        })
    }
}

/// Projects one Radar Directions leg onto the contract's route leg.
fn route_leg_from_radar(leg: RadarDirectionsLeg) -> EveryMapResult<RouteLeg> {
    let shape = leg
        .geometry
        .as_ref()
        .and_then(|geometry| geometry.polyline.as_deref())
        .filter(|polyline| !polyline.is_empty())
        .and_then(|polyline| {
            everymap_core::types::decode_encoded_polyline(polyline, RADAR_POLYLINE_PRECISION).ok()
        })
        .unwrap_or_default();
    let distance = leg
        .distance
        .value
        .ok_or_else(|| invalid_response("route leg distance lacks a value"))?;
    let duration = leg
        .duration
        .value
        .ok_or_else(|| invalid_response("route leg duration lacks a value"))?
        * SECONDS_PER_MINUTE;
    let steps = leg.steps.into_iter().map(route_step_from_radar).collect();
    Ok(RouteLeg {
        distance,
        duration,
        steps,
        shape,
    })
}

/// Projects one Radar Directions step onto the contract's route step.
fn route_step_from_radar(step: RadarDirectionsStep) -> RouteStep {
    RouteStep {
        instruction: step.instructions,
        distance: step.distance.value,
        duration: step.duration.value.map(|value| value * SECONDS_PER_MINUTE),
        start_coordinate: step
            .start_location
            .and_then(|location| Coordinate::new(location.latitude?, location.longitude?).ok()),
        end_coordinate: step
            .end_location
            .and_then(|location| Coordinate::new(location.latitude?, location.longitude?).ok()),
        maneuver_type: None,
        speech_instruction: None,
        lane_guidances: Vec::new(),
        shape: Vec::new(),
    }
}

#[async_trait]
impl everymap_core::domains::routing::Router for RadarRouter {
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        let locations = format!("{},{}|{},{}", start.lat, start.lng, end.lat, end.lng);
        let mut params: Vec<(&str, String)> = vec![("locations", locations)];

        if let Some(mode) = &options.transport_mode {
            params.push(("mode", map_transport_mode(mode).to_string()));
        }
        if let Some(avoid) = map_avoid_types(&options.avoid) {
            params.push(("avoid", avoid));
        }
        if let Some(alternatives) = options.alternatives {
            if alternatives > 0 {
                // Radar's alternatives flag is boolean and only applies to two-location requests
                params.push(("alternatives", "true".to_string()));
            }
        }
        if options.arrival_time.is_some() {
            log::warn!(
                "Radar Directions API does not support arrival_time; \
                 arrival_time will be ignored"
            );
        }

        // Extract Radar-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                    params.push(("units", v.to_string()));
                }
                if let Some(v) = obj.get("avoid").and_then(|v| v.as_str()) {
                    // provider_extra.avoid overrides the core avoid mapping
                    params.retain(|(key, _)| *key != "avoid");
                    params.push(("avoid", v.to_string()));
                }
                if let Some(v) = obj.get("geometry").and_then(|v| v.as_str()) {
                    params.push(("geometry", v.to_string()));
                }
                if let Some(v) = obj.get("heading").and_then(|v| v.as_f64()) {
                    params.push(("heading", v.to_string()));
                }
                if let Some(v) = obj.get("alternatives").and_then(|v| v.as_bool()) {
                    // provider_extra.alternatives overrides the core alternatives mapping
                    params.retain(|(key, _)| *key != "alternatives");
                    params.push(("alternatives", v.to_string()));
                }
                if let Some(v) = obj.get("lang").and_then(|v| v.as_str()) {
                    params.push(("lang", v.to_string()));
                }
            }
        }
        // Default geometry to polyline6 for decoding
        if !params.iter().any(|(k, _)| *k == "geometry") {
            params.push(("geometry", "polyline6".to_string()));
        }

        if let Some(departure_time) = &options.departure_time {
            params.push(("departureTime", departure_time.to_string()));
        }
        if let Some(lang) = &options.language {
            params.push(("lang", lang.clone()));
        }

        let url = self.directions_endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let radar_res: RadarDirectionsResponse = self.client.request_parsed_json(builder).await?;
        ensure_success(radar_res.meta.code)?;

        let routes = radar_res
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
                TransportMode::Truck,
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
    use crate::domain::types::{RadarGeometry, RadarLocation, RadarMetric};
    use types::{RadarDirectionsLeg, RadarDirectionsRoute, RadarDirectionsStep};

    #[test]
    fn test_map_transport_mode() {
        assert_eq!(map_transport_mode(&TransportMode::Car), "car");
        assert_eq!(map_transport_mode(&TransportMode::Truck), "truck");
        assert_eq!(map_transport_mode(&TransportMode::Pedestrian), "foot");
        assert_eq!(map_transport_mode(&TransportMode::Bicycle), "bike");
        assert_eq!(map_transport_mode(&TransportMode::Scooter), "foot");
    }

    #[test]
    fn test_map_avoid_types() {
        assert_eq!(
            map_avoid_types(&[AvoidType::Tolls, AvoidType::Highways]),
            Some("tolls,highways".to_string())
        );
        assert_eq!(
            map_avoid_types(&[AvoidType::Ferries]),
            Some("ferries".to_string())
        );
        // Unsupported types produce no features (warning is logged)
        assert!(map_avoid_types(&[AvoidType::Tunnels, AvoidType::DirtRoads]).is_none());
        assert!(map_avoid_types(&[]).is_none());
    }

    #[test]
    fn test_radar_directions_route_to_route_result() {
        let route = RadarDirectionsRoute {
            geometry: Some(RadarGeometry {
                polyline: Some("w{idcBgiooXfzj~Evd}_T".to_string()),
            }),
            distance: RadarMetric {
                value: Some(15000.0),
                text: "15 km".to_string(),
            },
            duration: RadarMetric {
                value: Some(25.0),
                text: "25 min".to_string(),
            },
            legs: vec![],
        };
        let result = RouteResult::try_from(route).expect("the fixture route converts");

        assert!((result.distance - 15000.0).abs() < f64::EPSILON);
        // Duration converted from minutes to seconds: 25 * 60 = 1500
        assert!((result.duration - 1500.0).abs() < f64::EPSILON);
        assert!(result.steps.is_empty());
        assert!(result.raw.is_some());
    }

    #[test]
    fn a_route_without_geometry_is_invalid() {
        let route = RadarDirectionsRoute {
            geometry: None,
            distance: RadarMetric {
                value: Some(5000.0),
                text: "5 km".to_string(),
            },
            duration: RadarMetric {
                value: Some(10.0),
                text: "10 min".to_string(),
            },
            legs: vec![],
        };
        match RouteResult::try_from(route) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn test_radar_directions_route_with_steps() {
        let step = RadarDirectionsStep {
            distance: RadarMetric {
                value: Some(200.0),
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: Some(0.5),
                text: "30 sec".to_string(),
            },
            start_location: Some(RadarLocation {
                latitude: Some(40.71),
                longitude: Some(-74.00),
            }),
            end_location: Some(RadarLocation {
                latitude: Some(40.72),
                longitude: Some(-74.01),
            }),
            bearing_before: 0.0,
            bearing_after: 90.0,
            instructions: Some("Turn left on Main St".to_string()),
            banner_instructions: None,
            voice_instructions: None,
            geometry: None,
            mode: None,
            maneuver: None,
            street_name: None,
            exit_name: None,
        };
        let leg = RadarDirectionsLeg {
            start_location: RadarLocation {
                latitude: Some(40.71),
                longitude: Some(-74.00),
            },
            end_location: RadarLocation {
                latitude: Some(40.72),
                longitude: Some(-74.01),
            },
            distance: RadarMetric {
                value: Some(200.0),
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: Some(0.5),
                text: "30 sec".to_string(),
            },
            geometry: None,
            steps: vec![step],
        };
        let route = RadarDirectionsRoute {
            geometry: Some(RadarGeometry {
                polyline: Some("w{idcBgiooXfzj~Evd}_T".to_string()),
            }),
            distance: RadarMetric {
                value: Some(200.0),
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: Some(0.5),
                text: "30 sec".to_string(),
            },
            legs: vec![leg],
        };
        let result = RouteResult::try_from(route).expect("the fixture route converts");

        assert_eq!(result.legs.len(), 1);
        assert_eq!(result.legs[0].steps.len(), 1);
        assert_eq!(result.steps.len(), 1);
        assert_eq!(
            result.steps[0].instruction,
            Some("Turn left on Main St".to_string())
        );
        assert_eq!(result.steps[0].distance, Some(200.0));
        // Duration for step also converted: 0.5 min * 60 = 30s
        assert_eq!(result.steps[0].duration, Some(30.0));
    }
}
