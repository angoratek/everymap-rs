pub mod types;

use crate::client::RadarClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    RouteOptions, RouteResponse, RouteResult, RouteStep, TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
pub use types::*;

const DIRECTIONS_BASE_URL: &str = "https://api.radar.io/v1/route/directions";

/// Implementation of `Router` for Radar Directions API.
pub struct RadarRouter {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
    pub(crate) distance_url: String,
    pub(crate) matrix_url: String,
}

impl RadarRouter {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: DIRECTIONS_BASE_URL.to_string(),
            distance_url: "https://api.radar.io/v1/route/distance".to_string(),
            matrix_url: "https://api.radar.io/v1/route/matrix".to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, base_url: String) -> Self {
        Self {
            client,
            base_url,
            distance_url: "https://api.radar.io/v1/route/distance".to_string(),
            matrix_url: "https://api.radar.io/v1/route/matrix".to_string(),
        }
    }

    pub fn with_ext_urls(
        client: std::sync::Arc<RadarClient>,
        base_url: String,
        distance_url: String,
        matrix_url: String,
    ) -> Self {
        Self {
            client,
            base_url,
            distance_url,
            matrix_url,
        }
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

impl From<RadarDirectionsRoute> for RouteResult {
    fn from(route: RadarDirectionsRoute) -> Self {
        let raw = serde_json::to_value(&route).unwrap_or_default();

        let distance = route.distance.value;
        let duration = route.duration.value * 60.0; // Radar returns minutes
        let geometry_str = route
            .geometry
            .as_ref()
            .and_then(|g| g.polyline.as_ref())
            .cloned()
            .unwrap_or_default();

        let steps: Vec<RouteStep> = route
            .legs
            .into_iter()
            .flat_map(|leg| leg.steps)
            .map(|s| RouteStep {
                instruction: s.instructions,
                distance: Some(s.distance.value),
                duration: Some(s.duration.value * 60.0), // Radar returns minutes
                start_coordinate: s.start_location.map(|l| {
                    Coordinate::new(l.latitude, l.longitude).unwrap_or(Coordinate::ORIGIN)
                }),
                end_coordinate: s.end_location.map(|l| {
                    Coordinate::new(l.latitude, l.longitude).unwrap_or(Coordinate::ORIGIN)
                }),
            })
            .collect();

        Self {
            distance,
            duration,
            geometry: everymap_core::types::Polyline::new(
                everymap_core::types::FlexiblePolyline::decode(&geometry_str).unwrap_or_default(),
            ),
            transport_mode: None,
            steps,
            bounding_box: None,
            raw: Some(raw),
        }
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
        if let Some(alternatives) = options.alternatives {
            log::warn!(
                "Radar Directions API does not support core alternatives field directly; \
                 use provider_extra.alternatives instead"
            );
            let _ = alternatives;
        }
        if !options.avoid.is_empty() {
            log::warn!(
                "Radar Directions API does not support core avoid field directly; \
                 use provider_extra.avoid instead"
            );
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
                    params.push(("avoid", v.to_string()));
                }
                if let Some(v) = obj.get("geometry").and_then(|v| v.as_str()) {
                    params.push(("geometry", v.to_string()));
                }
                if let Some(v) = obj.get("heading").and_then(|v| v.as_f64()) {
                    params.push(("heading", v.to_string()));
                }
                if let Some(v) = obj.get("alternatives").and_then(|v| v.as_bool()) {
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

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &self.base_url)
            .query(&params);

        let radar_res: RadarDirectionsResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!(
                    "Directions request failed with status {}",
                    radar_res.meta.code
                ),
            ));
        }

        let routes: Vec<RouteResult> = radar_res
            .routes
            .into_iter()
            .map(RouteResult::from)
            .collect();

        Ok(RouteResponse { routes })
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
    fn test_radar_directions_route_to_route_result() {
        let route = RadarDirectionsRoute {
            geometry: Some(RadarGeometry {
                polyline: Some("BFoz5xJ67i1B1B7PzIhacL2U1GE4".to_string()),
            }),
            distance: RadarMetric {
                value: 15000.0,
                text: "15 km".to_string(),
            },
            duration: RadarMetric {
                value: 25.0,
                text: "25 min".to_string(),
            },
            legs: vec![],
        };
        let result: RouteResult = route.into();

        assert!((result.distance - 15000.0).abs() < f64::EPSILON);
        // Duration converted from minutes to seconds: 25 * 60 = 1500
        assert!((result.duration - 1500.0).abs() < f64::EPSILON);
        assert!(result.steps.is_empty());
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_radar_directions_route_no_geometry() {
        let route = RadarDirectionsRoute {
            geometry: None,
            distance: RadarMetric {
                value: 5000.0,
                text: "5 km".to_string(),
            },
            duration: RadarMetric {
                value: 10.0,
                text: "10 min".to_string(),
            },
            legs: vec![],
        };
        let result: RouteResult = route.into();

        assert!((result.distance - 5000.0).abs() < f64::EPSILON);
        assert!((result.duration - 600.0).abs() < f64::EPSILON);
        assert!(result.geometry.points.is_empty());
    }

    #[test]
    fn test_radar_directions_route_with_steps() {
        let step = RadarDirectionsStep {
            distance: RadarMetric {
                value: 200.0,
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: 0.5,
                text: "30 sec".to_string(),
            },
            start_location: Some(RadarLocation {
                latitude: 40.71,
                longitude: -74.00,
            }),
            end_location: Some(RadarLocation {
                latitude: 40.72,
                longitude: -74.01,
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
                latitude: 40.71,
                longitude: -74.00,
            },
            end_location: RadarLocation {
                latitude: 40.72,
                longitude: -74.01,
            },
            distance: RadarMetric {
                value: 200.0,
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: 0.5,
                text: "30 sec".to_string(),
            },
            geometry: None,
            steps: vec![step],
        };
        let route = RadarDirectionsRoute {
            geometry: Some(RadarGeometry { polyline: None }),
            distance: RadarMetric {
                value: 200.0,
                text: "200 m".to_string(),
            },
            duration: RadarMetric {
                value: 0.5,
                text: "30 sec".to_string(),
            },
            legs: vec![leg],
        };
        let result: RouteResult = route.into();

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
