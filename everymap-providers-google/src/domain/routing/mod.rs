pub mod types;

use crate::client::{endpoint_url, GoogleClient};
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    DepartureTime, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult,
    RouteStep, Router, TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
pub use types::*;

/// Service root of the Google Routes API.
const ROUTES_SERVICE_ROOT: &str = "https://routes.googleapis.com";

/// Documented path of the Google Routes API `computeRoutes` method.
const ROUTES_PATH: &str = "/directions/v2:computeRoutes";

/// Required field mask header for Routes API v2 — requests without it fail.
const ROUTES_FIELD_MASK: &str = "routes.distanceMeters,routes.duration,routes.polyline.encodedPolyline,routes.viewport,routes.legs";

/// Implementation of `Router` for Google Maps Routes API v2.
pub struct GoogleRouter {
    pub(crate) client: std::sync::Arc<GoogleClient>,
    pub(crate) service_root: String,
}

impl GoogleRouter {
    pub fn new(client: std::sync::Arc<GoogleClient>) -> Self {
        Self {
            client,
            service_root: ROUTES_SERVICE_ROOT.to_string(),
        }
    }

    /// Creates a router against a Google Routes service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    /// - `service_root` — the scheme and host that serve the API, for example
    ///   `https://routes.googleapis.com` or a test server's origin. The
    ///   documented path is appended.
    pub fn with_base_url(client: std::sync::Arc<GoogleClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The endpoint URL this router sends requests to.
    fn endpoint(&self) -> String {
        endpoint_url(&self.service_root, ROUTES_PATH)
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

/// Number of decimal places in the encoded polyline the Routes API returns.
const GOOGLE_POLYLINE_PRECISION: u8 = 5;

impl TryFrom<GoogleRoute> for RouteResult {
    type Error = EveryMapError;

    fn try_from(route: GoogleRoute) -> Result<Self, Self::Error> {
        let duration = route
            .duration
            .as_deref()
            .and_then(parse_google_duration)
            .ok_or_else(|| invalid_response("route lacks a parseable duration"))?;

        let distance = route
            .distance_meters
            .ok_or_else(|| invalid_response("route lacks distanceMeters"))?
            as f64;

        let encoded = match route
            .polyline
            .as_ref()
            .and_then(|polyline| polyline.encoded_polyline.as_deref())
        {
            Some(encoded) if !encoded.is_empty() => encoded,
            _ => return Err(invalid_response("route lacks an encoded polyline")),
        };
        // The Routes API answers the classic Google encoding; the shared
        // decoder bounds the accumulator and refuses a malformed shape instead
        // of dropping points or shifting past the integer width.
        let geometry = Polyline::new(
            everymap_core::types::decode_encoded_polyline(encoded, GOOGLE_POLYLINE_PRECISION)
                .map_err(|detail| EveryMapError::provider("google", "INVALID_POLYLINE", detail))?,
        );

        let legs: Vec<RouteLeg> = route
            .legs
            .into_iter()
            .map(RouteLeg::try_from)
            .collect::<EveryMapResult<Vec<_>>>()?;
        let steps: Vec<RouteStep> = legs
            .iter()
            .flat_map(|leg| leg.steps.iter().cloned())
            .collect();

        let bounding_box = match route.viewport.as_ref() {
            Some(viewport) => {
                let north_east = viewport
                    .high
                    .as_ref()
                    .ok_or_else(|| invalid_response("route viewport lacks its north-east corner"))
                    .and_then(|lat_lng| {
                        Coordinate::new(lat_lng.latitude, lat_lng.longitude).map_err(|_| {
                            invalid_response("route viewport carries an invalid corner")
                        })
                    })?;
                let south_west = viewport
                    .low
                    .as_ref()
                    .ok_or_else(|| invalid_response("route viewport lacks its south-west corner"))
                    .and_then(|lat_lng| {
                        Coordinate::new(lat_lng.latitude, lat_lng.longitude).map_err(|_| {
                            invalid_response("route viewport carries an invalid corner")
                        })
                    })?;
                Some(BoundingBox::new(north_east, south_west))
            }
            None => None,
        };

        Ok(Self {
            distance,
            duration,
            geometry,
            // Filled from the request after conversion: the Routes API route
            // carries no `travelMode` field, and the mask cannot request one.
            transport_mode: None,
            steps,
            bounding_box,
            raw: None,
            legs,
        })
    }
}

impl TryFrom<GoogleRouteLeg> for RouteLeg {
    type Error = EveryMapError;

    fn try_from(leg: GoogleRouteLeg) -> Result<Self, Self::Error> {
        let distance = leg
            .distance_meters
            .ok_or_else(|| invalid_response("route leg lacks distanceMeters"))?
            as f64;
        let duration = leg
            .duration
            .as_deref()
            .and_then(parse_google_duration)
            .ok_or_else(|| invalid_response("route leg lacks a parseable duration"))?;
        let steps = leg.steps.into_iter().map(route_step_from_google).collect();
        Ok(Self {
            distance,
            duration,
            steps,
            shape: Vec::new(),
        })
    }
}

/// Projects one Routes API v2 step onto the contract's route step.
fn route_step_from_google(step: GoogleRouteStep) -> RouteStep {
    RouteStep {
        instruction: step
            .navigation_instruction
            .and_then(|instruction| instruction.instructions),
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
        maneuver_type: None,
        speech_instruction: None,
        lane_guidances: Vec::new(),
        shape: Vec::new(),
    }
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

        let url = self.endpoint();
        let builder = self
            .client
            .build_request(everymap_core::http::Method::POST, &url)
            .header("X-Goog-FieldMask", ROUTES_FIELD_MASK)
            .json(&request)?;

        let google_response: GoogleRoutesResponse =
            self.client.request_parsed_json(builder).await?;

        // The Routes API route carries no `travelMode` and the field mask
        // cannot request one, so the response echoes the requested mode.
        let routes = google_response
            .routes
            .into_iter()
            .map(|route| {
                let mut result = RouteResult::try_from(route)?;
                result.transport_mode = Some(transport_mode_from_google(travel_mode));
                Ok(result)
            })
            .collect::<everymap_core::error::EveryMapResult<Vec<_>>>()?;

        Ok(RouteResponse { routes })
    }

    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Taxi,
                TransportMode::Pedestrian,
                TransportMode::Bicycle,
                TransportMode::Scooter,
            ],
            // Routes API v2 ranks paths for travel time; it exposes no
            // distance-minimizing strategy.
            supports_shortest_route_strategy: false,
        }
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

    fn deserialize_route(value: serde_json::Value) -> GoogleRoute {
        serde_json::from_value(value).expect("the fixture deserializes")
    }

    #[test]
    fn a_route_without_a_duration_is_invalid() {
        let route = deserialize_route(serde_json::json!({
            "distanceMeters": 1000,
            "polyline": { "encodedPolyline": "_p~iF~ps|U" }
        }));
        match RouteResult::try_from(route) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn a_route_without_a_polyline_is_invalid() {
        let route = deserialize_route(serde_json::json!({
            "distanceMeters": 1000,
            "duration": "60s"
        }));
        match RouteResult::try_from(route) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }

    #[test]
    fn route_legs_carry_their_own_summary_and_steps() {
        let route = deserialize_route(serde_json::json!({
            "distanceMeters": 1000,
            "duration": "60s",
            "polyline": { "encodedPolyline": "_p~iF~ps|U" },
            "legs": [{
                "distanceMeters": 1000,
                "duration": "60s",
                "steps": [{
                    "distanceMeters": 1000,
                    "staticDuration": "60s",
                    "navigationInstruction": { "instructions": "Head north" }
                }]
            }]
        }));
        let result = RouteResult::try_from(route).expect("the fixture converts");
        assert_eq!(result.legs.len(), 1);
        assert_eq!(result.legs[0].distance, 1000.0);
        assert_eq!(result.legs[0].steps.len(), 1);
        assert_eq!(result.steps.len(), 1);
    }
}
