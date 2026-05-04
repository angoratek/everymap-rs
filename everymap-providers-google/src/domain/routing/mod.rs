pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    DepartureTime, RouteOptions, RouteResponse, RouteResult, RouteStep, Router, TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
pub use types::*;

const DIRECTIONS_BASE_URL: &str = "https://maps.googleapis.com/maps/api/directions/json";

/// Implementation of `Router` for Google Maps Directions API.
pub struct GoogleRouter {
    pub(crate) client: std::sync::Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleRouter {
    pub fn new(client: std::sync::Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: DIRECTIONS_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

impl From<GoogleRoute> for RouteResult {
    fn from(route: GoogleRoute) -> Self {
        let (distance, duration) = route
            .legs
            .first()
            .map(|leg| {
                let distance = leg.distance.as_ref().map(|d| d.value as f64).unwrap_or(0.0);
                let duration = leg.duration.as_ref().map(|d| d.value as f64).unwrap_or(0.0);
                (distance, duration)
            })
            .unwrap_or((0.0, 0.0));

        let geometry = route
            .overview_polyline
            .as_ref()
            .map(|p| decode_google_polyline(&p.points))
            .unwrap_or_else(|| Polyline::new(vec![]));

        let steps: Vec<RouteStep> =
            route
                .legs
                .first()
                .map(|leg| {
                    leg.steps
                        .iter()
                        .map(|s| RouteStep {
                            instruction: s.html_instructions.clone(),
                            distance: s.distance.as_ref().map(|d| d.value as f64),
                            duration: s.duration.as_ref().map(|d| d.value as f64),
                            start_coordinate: s.start_location.as_ref().map(|l| {
                                Coordinate::new(l.lat, l.lng).unwrap_or(Coordinate::ORIGIN)
                            }),
                            end_coordinate: s.end_location.as_ref().map(|l| {
                                Coordinate::new(l.lat, l.lng).unwrap_or(Coordinate::ORIGIN)
                            }),
                        })
                        .collect()
                })
                .unwrap_or_default();

        let bounding_box = route.bounds.map(|b| {
            BoundingBox::new(
                Coordinate::new(b.northeast.lat, b.northeast.lng).unwrap_or(Coordinate::ORIGIN),
                Coordinate::new(b.southwest.lat, b.southwest.lng).unwrap_or(Coordinate::ORIGIN),
            )
        });

        Self {
            distance,
            duration,
            geometry,
            transport_mode: None,
            steps,
            bounding_box,
            raw: None,
        }
    }
}

/// Decode a Google-encoded polyline (algorithm from Google Maps API docs).
fn decode_google_polyline(encoded: &str) -> Polyline {
    let mut points = Vec::new();
    let mut index = 0usize;
    let mut lat = 0i64;
    let mut lng = 0i64;

    while index < encoded.len() {
        let (dlat, new_index) = decode_polyline_value(encoded, index);
        lat += dlat;
        index = new_index;
        let (dlng, new_index) = decode_polyline_value(encoded, index);
        lng += dlng;
        index = new_index;

        let latitude = lat as f64 / 1e5;
        let longitude = lng as f64 / 1e5;
        if let Ok(coordinate) = Coordinate::new(latitude, longitude) {
            points.push(coordinate);
        }
    }

    Polyline::new(points)
}

fn decode_polyline_value(encoded: &str, mut index: usize) -> (i64, usize) {
    let mut result = 0i64;
    let mut shift = 0u32;
    let bytes = encoded.as_bytes();

    loop {
        if index >= bytes.len() {
            break;
        }
        let b = (bytes[index] as i64) - 63;
        index += 1;
        result |= (b & 0x1f) << shift;
        shift += 5;
        if b < 0x20 {
            break;
        }
    }

    if result & 1 != 0 {
        result = !(result >> 1);
    } else {
        result >>= 1;
    }

    (result, index)
}

#[async_trait]
impl Router for GoogleRouter {
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        let mode = match options.transport_mode {
            Some(TransportMode::Car) | None => "driving",
            Some(TransportMode::Bicycle) => "bicycling",
            Some(TransportMode::Pedestrian) => "walking",
            _ => "driving",
        };

        let mut params: Vec<(&str, String)> = vec![
            ("origin", format!("{},{}", start.lat, start.lng)),
            ("destination", format!("{},{}", end.lat, end.lng)),
            ("mode", mode.to_string()),
        ];

        if let Some(lang) = &options.language {
            params.push(("language", lang.clone()));
        }
        if !options.avoid.is_empty() {
            let avoid_str: String = options
                .avoid
                .iter()
                .filter_map(|a| match a {
                    everymap_core::domains::routing::AvoidType::Tolls => Some("tolls"),
                    everymap_core::domains::routing::AvoidType::Highways => Some("highways"),
                    everymap_core::domains::routing::AvoidType::Ferries => Some("ferries"),
                    // Tunnels and DirtRoads are not supported by Google Directions API
                    _ => None,
                })
                .collect::<Vec<&str>>()
                .join("|");
            if !avoid_str.is_empty() {
                params.push(("avoid", avoid_str));
            }
        }
        if let Some(alternatives) = options.alternatives {
            if alternatives > 1 {
                params.push(("alternatives", "true".to_string()));
            }
        }
        if let Some(departure_time) = &options.departure_time {
            let departure_value = match departure_time {
                DepartureTime::Now => "now".to_string(),
                DepartureTime::Timestamp(ts) => ts.to_string(),
                DepartureTime::Iso8601(s) => {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.timestamp().to_string())
                        .unwrap_or_else(|_| s.clone())
                }
            };
            params.push(("departure_time", departure_value));
        }

        // Extract Google-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("waypoints").and_then(|v| v.as_str()) {
                    // Google uses optimize:true| prefix in waypoints value for optimization
                    if let Some(optimize) = obj.get("optimize_waypoints").and_then(|v| v.as_bool()) {
                        if optimize {
                            params.push(("waypoints", format!("optimize:true|{}", v)));
                        } else {
                            params.push(("waypoints", v.to_string()));
                        }
                    } else {
                        params.push(("waypoints", v.to_string()));
                    }
                }
                if let Some(v) = obj.get("traffic_model").and_then(|v| v.as_str()) {
                    params.push(("traffic_model", v.to_string()));
                }
                if let Some(v) = obj.get("transit_mode").and_then(|v| v.as_str()) {
                    params.push(("transit_mode", v.to_string()));
                }
                if let Some(v) = obj
                    .get("transit_routing_preference")
                    .and_then(|v| v.as_str())
                {
                    params.push(("transit_routing_preference", v.to_string()));
                }
                if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                    params.push(("units", v.to_string()));
                }
            }
        }

        let url = self.base_url.clone();
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let google_res: GoogleDirectionsResponse = self.client.request_json(builder).await?;

        if google_res.status != "OK" && google_res.status != "ZERO_RESULTS" {
            return Err(EveryMapError::provider(
                "google",
                &google_res.status,
                google_res
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error"),
            ));
        }

        let routes: Vec<RouteResult> = google_res
            .routes
            .into_iter()
            .map(RouteResult::from)
            .collect();

        Ok(RouteResponse { routes })
    }
}
