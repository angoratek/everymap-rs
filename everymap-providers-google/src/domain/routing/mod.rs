pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    DepartureTime, RouteOptions, RouteResponse, RouteResult, RouteStep, Router, TransportMode,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
pub use types::*;

const ROUTES_BASE_URL: &str = "https://routes.googleapis.com/directions/v2:computeRoutes";

/// Required field mask header for Routes API v2 — requests without it fail.
const ROUTES_FIELD_MASK: &str = "routes.distanceMeters,routes.duration,routes.travelMode,routes.polyline.encodedPolyline,routes.viewport,routes.legs";

/// Implementation of `Router` for Google Maps Routes API v2.
pub struct GoogleRouter {
    pub(crate) client: std::sync::Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleRouter {
    pub fn new(client: std::sync::Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: ROUTES_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Resolve the Routes API v2 travel mode from core transport mode.
///
/// Falls back to `DRIVE` (with a warning) for modes Routes API v2 does not
/// support natively.
fn resolve_travel_mode(options: &RouteOptions) -> GoogleTravelMode {
    match options.transport_mode {
        None | Some(TransportMode::Car) | Some(TransportMode::Taxi) => GoogleTravelMode::Drive,
        Some(TransportMode::Bicycle) => GoogleTravelMode::Bicycle,
        Some(TransportMode::Pedestrian) => GoogleTravelMode::Walk,
        Some(TransportMode::Scooter) => GoogleTravelMode::TwoWheeler,
        Some(unsupported) => {
            log::warn!(
                "Google Routes API v2 does not support transport mode {:?}, falling back to DRIVE",
                unsupported
            );
            GoogleTravelMode::Drive
        }
    }
}

/// Map a Routes API v2 travel mode back to the core transport mode.
fn transport_mode_from_google(travel_mode: GoogleTravelMode) -> TransportMode {
    match travel_mode {
        GoogleTravelMode::Drive => TransportMode::Car,
        GoogleTravelMode::Walk => TransportMode::Pedestrian,
        GoogleTravelMode::Bicycle => TransportMode::Bicycle,
        GoogleTravelMode::TwoWheeler => TransportMode::Scooter,
        GoogleTravelMode::Transit => TransportMode::Bus,
    }
}

/// Build a Routes API v2 waypoint from a coordinate.
fn waypoint_from_coordinate(coordinate: &Coordinate) -> GoogleWaypoint {
    GoogleWaypoint {
        location: Some(GoogleRoutesLocation {
            lat_lng: GoogleRoutesLatLng {
                latitude: coordinate.lat,
                longitude: coordinate.lng,
            },
        }),
    }
}

/// Convert a `DepartureTime` to an RFC 3339 string for Routes API v2.
///
/// `DepartureTime::Now` maps to `None` (the API defaults to "now").
fn departure_time_string(departure_time: &DepartureTime) -> Option<String> {
    match departure_time {
        DepartureTime::Now => None,
        DepartureTime::Timestamp(timestamp) => chrono::DateTime::from_timestamp(*timestamp, 0)
            .map(|datetime| datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        DepartureTime::Iso8601(value) => Some(value.clone()),
    }
}

/// Parse a Google duration string like "1234s" into seconds.
fn parse_google_duration(duration: &str) -> Option<f64> {
    duration
        .strip_suffix('s')
        .and_then(|seconds| seconds.parse::<f64>().ok())
}

/// Build a waypoint from a "lat,lng" string (legacy provider_extra format).
fn waypoint_from_string(value: &str) -> Option<GoogleWaypoint> {
    let (latitude, longitude) = value.split_once(',')?;
    let latitude = latitude.trim().parse::<f64>().ok()?;
    let longitude = longitude.trim().parse::<f64>().ok()?;
    Coordinate::new(latitude, longitude)
        .ok()
        .map(|coordinate| waypoint_from_coordinate(&coordinate))
}

fn coordinate_from_location(location: &GoogleRoutesLocation) -> Option<Coordinate> {
    Coordinate::new(location.lat_lng.latitude, location.lat_lng.longitude).ok()
}

impl From<GoogleRoute> for RouteResult {
    fn from(route: GoogleRoute) -> Self {
        let duration = route
            .duration
            .as_deref()
            .and_then(parse_google_duration)
            .unwrap_or(0.0);

        let distance = route
            .distance_meters
            .map(|value| value as f64)
            .unwrap_or_else(|| {
                route
                    .legs
                    .first()
                    .and_then(|leg| leg.distance_meters)
                    .unwrap_or(0) as f64
            });

        let geometry = route
            .polyline
            .as_ref()
            .and_then(|polyline| polyline.encoded_polyline.as_deref())
            .map(decode_google_polyline)
            .unwrap_or_else(|| Polyline::new(vec![]));

        let steps: Vec<RouteStep> = route
            .legs
            .first()
            .map(|leg| {
                leg.steps
                    .iter()
                    .map(|step| RouteStep {
                        instruction: step
                            .navigation_instruction
                            .as_ref()
                            .and_then(|instruction| instruction.instructions.clone()),
                        distance: step.distance_meters.map(|value| value as f64),
                        duration: step
                            .static_duration
                            .as_deref()
                            .and_then(parse_google_duration),
                        start_coordinate: step
                            .start_location
                            .as_ref()
                            .and_then(coordinate_from_location),
                        end_coordinate: step
                            .end_location
                            .as_ref()
                            .and_then(coordinate_from_location),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let bounding_box =
            route.viewport.as_ref().and_then(|viewport| {
                let north_east = viewport.high.as_ref().and_then(|lat_lng| {
                    Coordinate::new(lat_lng.latitude, lat_lng.longitude).ok()
                })?;
                let south_west = viewport.low.as_ref().and_then(|lat_lng| {
                    Coordinate::new(lat_lng.latitude, lat_lng.longitude).ok()
                })?;
                Some(BoundingBox::new(north_east, south_west))
            });

        Self {
            distance,
            duration,
            geometry,
            transport_mode: route.travel_mode.map(transport_mode_from_google),
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
        let travel_mode = resolve_travel_mode(options);

        let mut request = GoogleRoutesRequest {
            origin: waypoint_from_coordinate(start),
            destination: waypoint_from_coordinate(end),
            intermediates: Vec::new(),
            travel_mode: Some(travel_mode),
            compute_alternative_routes: None,
            route_modifiers: None,
            language_code: None,
            departure_time: None,
            optimize_waypoints: None,
            units: None,
        };

        if let Some(language) = &options.language {
            request.language_code = Some(language.clone());
        }
        if let Some(alternatives) = options.alternatives {
            if alternatives > 1 {
                request.compute_alternative_routes = Some(true);
            }
        }

        if let Some(departure_time) = &options.departure_time {
            request.departure_time = departure_time_string(departure_time);
        }

        if options.arrival_time.is_some() {
            log::warn!(
                "Google Routes API v2 only supports arrivalTime with TRANSIT travel mode; \
                 arrival_time will be ignored"
            );
        }

        if !options.avoid.is_empty() {
            let mut modifiers = GoogleRouteModifiers::default();
            for avoid_type in &options.avoid {
                match avoid_type {
                    everymap_core::domains::routing::AvoidType::Tolls => {
                        modifiers.avoid_tolls = true;
                    }
                    everymap_core::domains::routing::AvoidType::Ferries => {
                        modifiers.avoid_ferries = true;
                    }
                    everymap_core::domains::routing::AvoidType::Highways => {
                        modifiers.avoid_highways = true;
                    }
                    everymap_core::domains::routing::AvoidType::Tunnels => {
                        log::warn!(
                            "Google Routes API v2 only supports avoidTunnels with \
                             routingPreference TRAFFIC_AWARE_OPTIMAL; ignoring"
                        );
                    }
                    unsupported => {
                        log::warn!(
                            "Google Routes API v2 does not support avoid type {:?}; ignoring",
                            unsupported
                        );
                    }
                }
            }
            if modifiers.avoid_tolls || modifiers.avoid_ferries || modifiers.avoid_highways {
                request.route_modifiers = Some(modifiers);
            }
        }

        // Extract Google-specific options from provider_extra
        if let Some(extra) = &options.provider_extra {
            if let Some(object) = extra.as_object() {
                if let Some(value) = object.get("waypoints").and_then(|v| v.as_str()) {
                    for waypoint in value.split('|') {
                        match waypoint_from_string(waypoint) {
                            Some(waypoint) => request.intermediates.push(waypoint),
                            None => log::warn!(
                                "Google Routes API v2 could not parse waypoint {:?}; ignoring",
                                waypoint
                            ),
                        }
                    }
                }
                if let Some(value) = object.get("optimize_waypoints").and_then(|v| v.as_bool()) {
                    request.optimize_waypoints = Some(value);
                }
                if let Some(value) = object.get("units").and_then(|v| v.as_str()) {
                    match value.to_ascii_lowercase().as_str() {
                        "metric" => request.units = Some("METRIC".to_string()),
                        "imperial" => request.units = Some("IMPERIAL".to_string()),
                        other => log::warn!(
                            "Google Routes API v2 does not support units {:?}; ignoring",
                            other
                        ),
                    }
                }
                for legacy_field in [
                    "traffic_model",
                    "transit_mode",
                    "transit_routing_preference",
                ] {
                    if object.contains_key(legacy_field) {
                        log::warn!(
                            "Google Routes API v2 does not support provider_extra field {:?}; ignoring",
                            legacy_field
                        );
                    }
                }
            }
        }

        let url = self.base_url.clone();
        let builder = self
            .client
            .build_request(reqwest::Method::POST, &url)
            .header("X-Goog-FieldMask", ROUTES_FIELD_MASK)
            .json(&request);

        let google_response: GoogleRoutesResponse = self.client.request_json(builder).await?;

        let routes: Vec<RouteResult> = google_response
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

    #[test]
    fn test_parse_google_duration() {
        assert_eq!(parse_google_duration("1234s"), Some(1234.0));
        assert_eq!(parse_google_duration("0s"), Some(0.0));
        assert_eq!(parse_google_duration("12.5s"), Some(12.5));
        assert_eq!(parse_google_duration("1234"), None);
        assert_eq!(parse_google_duration("abc"), None);
    }

    #[test]
    fn test_departure_time_string_timestamp() {
        let value = departure_time_string(&DepartureTime::Timestamp(1717200000));
        assert_eq!(value.as_deref(), Some("2024-06-01T00:00:00Z"));
    }

    #[test]
    fn test_departure_time_string_now_is_none() {
        assert_eq!(departure_time_string(&DepartureTime::Now), None);
    }

    #[test]
    fn test_departure_time_string_iso8601_passthrough() {
        let value =
            departure_time_string(&DepartureTime::Iso8601("2024-06-01T08:00:00Z".to_string()));
        assert_eq!(value.as_deref(), Some("2024-06-01T08:00:00Z"));
    }
}
