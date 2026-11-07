//! Valhalla route calculation: request encoding, response parsing and the
//! [`Router`] implementation.
//!
//! Valhalla is a self-hosted routing engine. Its `/route` endpoint accepts a
//! JSON POST and answers with a `trip` object whose legs carry maneuvers,
//! summaries and precision-6 encoded shapes. This module maps the
//! vendor-neutral [`RouteOptions`] onto Valhalla's `costing` vocabulary and
//! normalizes every distance to meters and every duration to seconds.
//!
//! # Option mapping
//!
//! - **Transport mode.** [`TransportMode::Car`] selects the `auto` costing,
//!   [`TransportMode::Bicycle`] the `bicycle` costing and
//!   [`TransportMode::Pedestrian`] the `pedestrian` costing. A mode none of
//!   those costings covers falls back to `auto` with a warning.
//! - **Units.** Requests always ask for kilometers, and the parser converts
//!   the answer to meters.
//! - **Avoidance.** [`AvoidType::Tolls`] and [`AvoidType::Highways`] become
//!   the automobile cost factors `use_tolls` and `use_highways`.
//!   [`AvoidType::DirtRoads`] becomes `use_tracks` for automobile and bicycle;
//!   pedestrian costing has no unpaved-road factor.
//!   [`AvoidType::Ferries`] becomes `use_ferry` in every mode. Valhalla has no
//!   cost factor for tunnels, so [`AvoidType::Tunnels`] changes nothing.
//! - **Strategy.** [`RouteStrategy::Shortest`] sets the automobile `shortest`
//!   cost factor. Non-automobile costing has no time-versus-distance choice.
//! - **Options not forwarded.** `departure_time`, `arrival_time`, `language`
//!   and `alternatives` are not sent; Valhalla's `date_time` expects an
//!   ISO-8601 instant with a routing time zone, which the route contract does
//!   not carry.
//! - **Lane guidance.** This crate does not map Valhalla's lane records onto
//!   the contract's [`LaneGuidance`] vocabulary, so every step's lane list is
//!   empty.

use crate::client::{ValhallaClient, ValhallaProviderConfiguration};
use crate::error_mapping::{
    invalid_response, provider_error_from_body, provider_error_from_payload,
};
use crate::maneuver_mapping::map_valhalla_maneuver_type;
use crate::polyline::decode_polyline6;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    AvoidType, LaneGuidance, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult,
    RouteStep, RouteStrategy, Router, TransportMode,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpRequest, HttpTransport, Method};
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Smallest number of waypoints a route can connect.
const MINIMUM_ROUTE_WAYPOINT_COUNT: usize = 2;

/// Conversion factor from the kilometers Valhalla reports to the meters the
/// contract carries.
const KILOMETERS_TO_METERS: f64 = 1_000.0;

/// Unit system every request asks Valhalla to report.
const VALHALLA_UNITS_KILOMETERS: &str = "kilometers";

/// Path of the route calculation endpoint below the configured base URL.
const VALHALLA_ROUTE_PATH: &str = "route";

/// Header identifying the calling application to the server.
const USER_AGENT_HEADER_NAME: &str = "user-agent";

/// First HTTP status that signals a server-side outage. A response in this
/// range keeps the shared client's HTTP error rather than a provider refusal.
const HTTP_SERVER_ERROR_STATUS_START: u16 = 500;

/// Valhalla costing for private car routing.
const COSTING_AUTOMOBILE: &str = "auto";

/// Valhalla costing for bicycle routing.
const COSTING_BICYCLE: &str = "bicycle";

/// Valhalla costing for walking routing.
const COSTING_PEDESTRIAN: &str = "pedestrian";

/// Automobile cost factor selecting the shortest path rather than the
/// fastest one.
const COSTING_OPTION_SHORTEST: &str = "shortest";

/// Cost factor discouraging toll roads.
const COSTING_OPTION_USE_TOLLS: &str = "use_tolls";

/// Cost factor discouraging limited-access highways.
const COSTING_OPTION_USE_HIGHWAYS: &str = "use_highways";

/// Cost factor discouraging ferry crossings.
const COSTING_OPTION_USE_FERRY: &str = "use_ferry";

/// Cost factor discouraging unpaved tracks.
const COSTING_OPTION_USE_TRACKS: &str = "use_tracks";

/// Cost factor value that fully disallows a feature when the costing model
/// allows it.
const COSTING_FACTOR_DISALLOW: f64 = 0.0;

/// Routing through a Valhalla server's `/route` endpoint.
pub struct ValhallaRouter {
    client: Arc<ValhallaClient>,
    base_url: String,
    user_agent: String,
}

impl ValhallaRouter {
    /// Builds a router for one Valhalla server on the target's default
    /// transport.
    ///
    /// # Parameters
    /// - `configuration` — the server base URL and user agent.
    ///
    /// # Returns
    /// A router ready to calculate routes.
    pub fn new(configuration: ValhallaProviderConfiguration) -> Self {
        Self::with_client(Arc::new(ValhallaClient::keyless()), configuration)
    }

    /// Builds a router that publishes each attempt to `observer`.
    ///
    /// # Parameters
    /// - `configuration` — the server base URL and user agent.
    /// - `observer` — receives every request attempt and its outcome.
    ///
    /// # Returns
    /// A router ready to calculate routes and report each attempt.
    pub fn with_observer(
        configuration: ValhallaProviderConfiguration,
        observer: Arc<dyn everymap_core::client::RequestObserver>,
    ) -> Self {
        let mut client = ValhallaClient::keyless();
        client.set_observer(observer);
        Self::with_client(Arc::new(client), configuration)
    }

    /// Builds a router that executes requests through an explicit transport.
    ///
    /// # Parameters
    /// - `transport` — the transport that sends each request. Hosts inject
    ///   their platform transport and tests inject a fake.
    /// - `configuration` — the server base URL and user agent.
    ///
    /// # Returns
    /// A router ready to calculate routes through the supplied transport.
    pub fn with_transport(
        transport: Arc<dyn HttpTransport>,
        configuration: ValhallaProviderConfiguration,
    ) -> Self {
        Self::with_client(
            Arc::new(ValhallaClient::with_transport(transport)),
            configuration,
        )
    }

    /// Builds a router around a client; the base URL is joined with the route
    /// path at request time.
    fn with_client(
        client: Arc<ValhallaClient>,
        configuration: ValhallaProviderConfiguration,
    ) -> Self {
        Self {
            client,
            base_url: configuration.base_url,
            user_agent: configuration.user_agent,
        }
    }

    /// Calculates one route for the ordered waypoints and returns the trip.
    ///
    /// # Parameters
    /// - `waypoints` — at least two ordered stops.
    /// - `options` — transport mode, strategy, avoidance and timing choices.
    ///
    /// # Returns
    /// A response with one route whose distances are meters and whose
    /// durations are seconds.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] for fewer than two waypoints.
    /// - [`EveryMapError::ProviderError`] for a Valhalla refusal or a
    ///   structurally unusable payload.
    /// - [`EveryMapError::HttpError`] for a server error or a non-success
    ///   status without a Valhalla envelope.
    /// - [`EveryMapError::SerializationError`] for a body that is not valid
    ///   JSON or does not carry a trip.
    /// - [`EveryMapError::RateLimited`] for HTTP 429, as the shared client
    ///   reports it.
    /// - [`EveryMapError::TransportError`] or [`EveryMapError::ClientError`]
    ///   when no response arrives.
    async fn request_route(
        &self,
        waypoints: &[Coordinate],
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        if waypoints.len() < MINIMUM_ROUTE_WAYPOINT_COUNT {
            return Err(EveryMapError::ValidationError(format!(
                "a Valhalla route needs at least {MINIMUM_ROUTE_WAYPOINT_COUNT} waypoints; got {}",
                waypoints.len()
            )));
        }
        let transport_mode = options.transport_mode.unwrap_or(TransportMode::Car);
        let request_body = build_route_request_body(waypoints, options, transport_mode)?;
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            VALHALLA_ROUTE_PATH
        );
        let mut request = self
            .client
            .build_request(Method::Post, &url)
            .json(&request_body)?;
        if !self.user_agent.is_empty() {
            request = request.header(USER_AGENT_HEADER_NAME, &self.user_agent);
        }
        let body_text = self.send(request).await?;
        let payload: Value = serde_json::from_str(&body_text)?;
        if let Some(provider_error) = provider_error_from_payload(&payload) {
            return Err(provider_error);
        }
        let route = parse_route_payload(payload, transport_mode)?;
        Ok(RouteResponse {
            routes: vec![route],
        })
    }

    /// Sends one request and returns the body of a successful response.
    ///
    /// A `4xx` response that carries a Valhalla error object becomes a
    /// provider error so the service's own code and message survive; a `5xx`
    /// response and any other non-success response keep the shared client's
    /// HTTP error.
    async fn send(&self, request: HttpRequest) -> EveryMapResult<String> {
        match self.client.request(request).await {
            Ok(response) => Ok(response.text()),
            Err(EveryMapError::HttpError {
                status,
                message,
                body,
            }) => {
                if status < HTTP_SERVER_ERROR_STATUS_START {
                    if let Some(provider_error) = body.as_deref().and_then(provider_error_from_body)
                    {
                        return Err(provider_error);
                    }
                }
                Err(EveryMapError::HttpError {
                    status,
                    message,
                    body,
                })
            }
            Err(error) => Err(error),
        }
    }
}

#[async_trait]
impl Router for ValhallaRouter {
    /// Calculates a route between two endpoints.
    ///
    /// # Parameters
    /// - `start` — the origin.
    /// - `end` — the destination.
    /// - `options` — transport mode, strategy, avoidance and timing choices.
    ///
    /// # Returns
    /// A response with one route.
    ///
    /// # Errors
    /// See [`Router::calculate_route_with_waypoints`].
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        self.request_route(&[*start, *end], options).await
    }

    /// Calculates a route through ordered waypoints.
    ///
    /// # Parameters
    /// - `waypoints` — at least two ordered stops; consecutive pairs form the
    ///   returned legs.
    /// - `options` — transport mode, strategy, avoidance and timing choices.
    ///
    /// # Returns
    /// A response with one route and one leg per consecutive waypoint pair.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] for fewer than two
    /// waypoints; the private `request_route` carries the request failures.
    async fn calculate_route_with_waypoints(
        &self,
        waypoints: &[Coordinate],
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        self.request_route(waypoints, options).await
    }

    /// Valhalla's costings cover car, bicycle and pedestrian routing, and the
    /// automobile costing carries the shortest-path cost factor.
    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Bicycle,
                TransportMode::Pedestrian,
            ],
            supports_shortest_route_strategy: true,
        }
    }
}

/// One location in a Valhalla request; latitude precedes longitude.
#[derive(Debug, Serialize)]
struct ValhallaLocation {
    /// Latitude in decimal degrees.
    lat: f64,
    /// Longitude in decimal degrees.
    lon: f64,
}

/// The `directions_options` block of a Valhalla request.
#[derive(Debug, Serialize)]
struct ValhallaDirectionsOptions {
    /// Unit system used in the returned instructions and lengths.
    units: &'static str,
}

/// The complete body of a Valhalla `/route` request.
#[derive(Debug, Serialize)]
struct ValhallaRouteRequest {
    /// Ordered waypoints, origin first.
    locations: Vec<ValhallaLocation>,
    /// Valhalla costing model name.
    costing: &'static str,
    /// Unit system used for the returned distances.
    units: &'static str,
    /// Instruction options.
    directions_options: ValhallaDirectionsOptions,
    /// Cost factors; omitted when no requested option needs one.
    #[serde(skip_serializing_if = "Option::is_none")]
    costing_options: Option<Value>,
}

/// Encodes one route request.
///
/// # Parameters
/// - `waypoints` — the ordered stops; the caller guarantees at least two.
/// - `options` — the options whose expressive subset Valhalla accepts.
/// - `transport_mode` — the mode the request routes for.
///
/// # Returns
/// The JSON request object, with locations in latitude/longitude order.
///
/// # Errors
/// Returns [`EveryMapError::SerializationError`] when the request cannot be
/// serialized.
fn build_route_request_body(
    waypoints: &[Coordinate],
    options: &RouteOptions,
    transport_mode: TransportMode,
) -> EveryMapResult<Value> {
    let request = ValhallaRouteRequest {
        locations: waypoints
            .iter()
            .map(|waypoint| ValhallaLocation {
                lat: waypoint.lat,
                lon: waypoint.lng,
            })
            .collect(),
        costing: costing_text(transport_mode),
        units: VALHALLA_UNITS_KILOMETERS,
        directions_options: ValhallaDirectionsOptions {
            units: VALHALLA_UNITS_KILOMETERS,
        },
        costing_options: costing_options(options, transport_mode),
    };
    Ok(serde_json::to_value(request)?)
}

/// Names the Valhalla costing model for a transport mode.
fn costing_text(transport_mode: TransportMode) -> &'static str {
    match transport_mode {
        TransportMode::Car => COSTING_AUTOMOBILE,
        TransportMode::Bicycle => COSTING_BICYCLE,
        TransportMode::Pedestrian => COSTING_PEDESTRIAN,
        unsupported => {
            log::warn!(
                "Valhalla routing does not support transport mode {unsupported:?}, falling back to the automobile costing"
            );
            COSTING_AUTOMOBILE
        }
    }
}

/// Builds the `costing_options` block for the requested strategy and
/// avoidance features, or `None` when no requested option needs one.
///
/// The keys are mode-specific because Valhalla validates them against the
/// selected costing model: an automobile factor sent with pedestrian costing
/// is a server-side error, not a silently ignored preference. The flags
/// Valhalla cannot express in a mode change nothing there.
fn costing_options(options: &RouteOptions, transport_mode: TransportMode) -> Option<Value> {
    let mut mode_options = serde_json::Map::new();
    if transport_mode == TransportMode::Car
        && options.route_strategy == Some(RouteStrategy::Shortest)
    {
        mode_options.insert(COSTING_OPTION_SHORTEST.to_string(), Value::Bool(true));
    }
    if transports_automobile(transport_mode) && avoid_type_requested(options, AvoidType::Tolls) {
        mode_options.insert(
            COSTING_OPTION_USE_TOLLS.to_string(),
            serde_json::json!(COSTING_FACTOR_DISALLOW),
        );
    }
    if transports_automobile(transport_mode) && avoid_type_requested(options, AvoidType::Highways) {
        mode_options.insert(
            COSTING_OPTION_USE_HIGHWAYS.to_string(),
            serde_json::json!(COSTING_FACTOR_DISALLOW),
        );
    }
    if avoid_type_requested(options, AvoidType::Ferries) {
        mode_options.insert(
            COSTING_OPTION_USE_FERRY.to_string(),
            serde_json::json!(COSTING_FACTOR_DISALLOW),
        );
    }
    if transport_mode != TransportMode::Pedestrian
        && avoid_type_requested(options, AvoidType::DirtRoads)
    {
        mode_options.insert(
            COSTING_OPTION_USE_TRACKS.to_string(),
            serde_json::json!(COSTING_FACTOR_DISALLOW),
        );
    }
    if mode_options.is_empty() {
        return None;
    }
    let mut costing_options = serde_json::Map::new();
    costing_options.insert(
        costing_text(transport_mode).to_string(),
        Value::Object(mode_options),
    );
    Some(Value::Object(costing_options))
}

/// Whether the mode uses the automobile costing.
///
/// Every mode the crate does not name falls back to `auto`, so it accepts the
/// automobile cost factors as well.
fn transports_automobile(transport_mode: TransportMode) -> bool {
    match transport_mode {
        TransportMode::Bicycle | TransportMode::Pedestrian => false,
        TransportMode::Car
        | TransportMode::Truck
        | TransportMode::Scooter
        | TransportMode::Bus
        | TransportMode::Taxi
        | TransportMode::Unknown => true,
    }
}

/// Whether the request asked to avoid one feature.
fn avoid_type_requested(options: &RouteOptions, avoid_type: AvoidType) -> bool {
    options.avoid.contains(&avoid_type)
}

/// A `trip` response envelope.
#[derive(Debug, Deserialize)]
struct ValhallaTripEnvelope {
    /// The calculated trip.
    trip: ValhallaTrip,
}

/// One calculated trip.
#[derive(Debug, Deserialize)]
struct ValhallaTrip {
    /// Trip-wide totals.
    summary: ValhallaSummary,
    /// One leg per consecutive waypoint pair.
    legs: Vec<ValhallaLeg>,
}

/// Distance and duration totals, in kilometers and seconds.
#[derive(Debug, Deserialize)]
struct ValhallaSummary {
    /// Total length in kilometers.
    length: f64,
    /// Total travel time in seconds.
    time: f64,
}

/// One leg between consecutive waypoints.
#[derive(Debug, Deserialize)]
struct ValhallaLeg {
    /// Leg totals.
    summary: ValhallaSummary,
    /// Precision-6 encoded leg geometry.
    shape: String,
    /// Maneuvers in traversal order.
    maneuvers: Vec<ValhallaManeuver>,
}

/// One maneuver within a leg.
#[derive(Debug, Deserialize)]
struct ValhallaManeuver {
    /// Valhalla maneuver action integer.
    #[serde(rename = "type")]
    maneuver_type: i32,
    /// Display instruction.
    instruction: String,
    /// Text-to-speech instruction; absent for some maneuvers.
    verbal_pre_transition_instruction: Option<String>,
    /// Maneuver length in kilometers.
    length: f64,
    /// Maneuver travel time in seconds.
    time: f64,
    /// First shape index of the maneuver.
    begin_shape_index: Option<usize>,
    /// Last shape index of the maneuver.
    end_shape_index: Option<usize>,
    /// Roundabout exit number when the server reports one.
    exit_number: Option<u8>,
    /// Roundabout exit count when the server reports one instead.
    roundabout_exit_count: Option<u8>,
}

/// Parses a Valhalla `trip` payload into a [`RouteResult`].
///
/// # Parameters
/// - `payload` — the decoded JSON body of a successful `/route` response.
/// - `transport_mode` — the mode the request asked for, echoed on the result.
///
/// # Returns
/// The route with meters, seconds, decoded step shapes, one leg per
/// consecutive waypoint pair and a bounding box over the decoded geometry.
///
/// # Errors
/// - [`EveryMapError::SerializationError`] when the payload is not a trip.
/// - [`EveryMapError::ProviderError`] with the `INVALID_RESPONSE` code when the
///   trip carries no legs or a leg carries an empty shape, and with the
///   `INVALID_POLYLINE` code when a shape cannot be decoded.
fn parse_route_payload(
    payload: Value,
    transport_mode: TransportMode,
) -> EveryMapResult<RouteResult> {
    let envelope: ValhallaTripEnvelope = serde_json::from_value(payload)?;
    let ValhallaTrip { summary, legs } = envelope.trip;
    if legs.is_empty() {
        return Err(invalid_response("Valhalla trip carries no legs"));
    }
    let mut route_legs = Vec::with_capacity(legs.len());
    let mut route_steps = Vec::new();
    let mut route_coordinates = Vec::new();
    for leg in legs {
        let leg_shape = decode_polyline6(&leg.shape)?;
        if leg_shape.is_empty() {
            return Err(invalid_response("Valhalla trip leg carries an empty shape"));
        }
        let steps = leg
            .maneuvers
            .into_iter()
            .map(|maneuver| maneuver_to_step(maneuver, &leg_shape))
            .collect::<Vec<RouteStep>>();
        route_steps.extend(steps.iter().cloned());
        route_coordinates.extend_from_slice(&leg_shape);
        route_legs.push(RouteLeg {
            distance: kilometers_to_meters(leg.summary.length),
            duration: leg.summary.time,
            steps,
            shape: leg_shape,
        });
    }
    let bounding_box = bounding_box_from_coordinates(&route_coordinates)?;
    Ok(RouteResult {
        distance: kilometers_to_meters(summary.length),
        duration: summary.time,
        geometry: Polyline::new(route_coordinates),
        transport_mode: Some(transport_mode),
        steps: route_steps,
        bounding_box: Some(bounding_box),
        raw: None,
        legs: route_legs,
    })
}

/// Converts one Valhalla maneuver into a contract step.
///
/// # Parameters
/// - `maneuver` — the maneuver record.
/// - `leg_shape` — the decoded geometry the maneuver's shape indexes refer
///   to.
///
/// # Returns
/// The contract step for the maneuver.
fn maneuver_to_step(maneuver: ValhallaManeuver, leg_shape: &[Coordinate]) -> RouteStep {
    let roundabout_exit_index = maneuver.exit_number.or(maneuver.roundabout_exit_count);
    let speech_instruction = maneuver
        .verbal_pre_transition_instruction
        .unwrap_or_else(|| maneuver.instruction.clone());
    let shape = maneuver_shape_slice(
        leg_shape,
        maneuver.begin_shape_index,
        maneuver.end_shape_index,
    );
    RouteStep {
        instruction: Some(maneuver.instruction),
        distance: Some(kilometers_to_meters(maneuver.length)),
        duration: Some(maneuver.time),
        start_coordinate: shape.first().copied(),
        end_coordinate: shape.last().copied(),
        maneuver_type: Some(map_valhalla_maneuver_type(
            maneuver.maneuver_type,
            roundabout_exit_index,
        )),
        speech_instruction: Some(speech_instruction),
        // Valhalla does not return the contract's per-lane guidance record.
        lane_guidances: Vec::<LaneGuidance>::new(),
        shape,
    }
}

/// Slices the leg geometry covered by one maneuver.
///
/// # Parameters
/// - `leg_shape` — the decoded leg geometry.
/// - `begin_shape_index` — the maneuver's first shape index.
/// - `end_shape_index` — the maneuver's last shape index.
///
/// # Returns
/// The inclusive slice, clamped to the geometry; an absent or inverted index
/// pair yields an empty shape rather than a panic.
fn maneuver_shape_slice(
    leg_shape: &[Coordinate],
    begin_shape_index: Option<usize>,
    end_shape_index: Option<usize>,
) -> Vec<Coordinate> {
    let (Some(begin_shape_index), Some(end_shape_index)) = (begin_shape_index, end_shape_index)
    else {
        return Vec::new();
    };
    if begin_shape_index >= leg_shape.len() || end_shape_index < begin_shape_index {
        return Vec::new();
    }
    let inclusive_end_index = end_shape_index.min(leg_shape.len() - 1);
    leg_shape[begin_shape_index..=inclusive_end_index].to_vec()
}

/// Computes the bounding box that encloses every decoded geometry point.
///
/// # Parameters
/// - `coordinates` — all decoded leg points.
///
/// # Returns
/// The south-west and north-east corners of the route.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] with the `INVALID_RESPONSE` code
/// when there is no geometry or a corner is outside the valid ranges.
fn bounding_box_from_coordinates(coordinates: &[Coordinate]) -> EveryMapResult<BoundingBox> {
    let Some(first_coordinate) = coordinates.first() else {
        return Err(invalid_response("Valhalla trip carries no geometry"));
    };
    let mut minimum_latitude = first_coordinate.lat;
    let mut minimum_longitude = first_coordinate.lng;
    let mut maximum_latitude = first_coordinate.lat;
    let mut maximum_longitude = first_coordinate.lng;
    for coordinate in coordinates {
        minimum_latitude = minimum_latitude.min(coordinate.lat);
        minimum_longitude = minimum_longitude.min(coordinate.lng);
        maximum_latitude = maximum_latitude.max(coordinate.lat);
        maximum_longitude = maximum_longitude.max(coordinate.lng);
    }
    let south_west = Coordinate::new(minimum_latitude, minimum_longitude)
        .map_err(|error| invalid_response(format!("Valhalla route bounds are invalid: {error}")))?;
    let north_east = Coordinate::new(maximum_latitude, maximum_longitude)
        .map_err(|error| invalid_response(format!("Valhalla route bounds are invalid: {error}")))?;
    Ok(BoundingBox::new(north_east, south_west))
}

/// Converts a Valhalla length in kilometers to the contract's meters.
fn kilometers_to_meters(kilometers: f64) -> f64 {
    kilometers * KILOMETERS_TO_METERS
}

#[cfg(test)]
mod tests {
    use super::{build_route_request_body, parse_route_payload};
    use everymap_core::domains::routing::{
        AvoidType, ManeuverType, RouteOptions, RouteStrategy, TransportMode, TurnDirection,
    };
    use everymap_core::error::EveryMapError;
    use everymap_core::types::Coordinate;
    use serde_json::{json, Value};

    /// Tolerance for distances and durations asserted on the fixture.
    const FIXTURE_NUMERIC_TOLERANCE: f64 = 1e-6;

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    fn coordinate(latitude: f64, longitude: f64) -> Coordinate {
        Coordinate::new(latitude, longitude).unwrap()
    }

    fn recorded_waypoints() -> Vec<Coordinate> {
        vec![
            coordinate(38.57657, -121.4936),
            coordinate(38.57887, -121.50797),
        ]
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < FIXTURE_NUMERIC_TOLERANCE,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn request_carries_locations_in_latitude_longitude_order() {
        let body = build_route_request_body(
            &recorded_waypoints(),
            &RouteOptions::default(),
            TransportMode::Car,
        )
        .expect("the request must encode");
        assert_eq!(
            body,
            json!({
                "locations": [
                    {"lat": 38.57657, "lon": -121.4936},
                    {"lat": 38.57887, "lon": -121.50797}
                ],
                "costing": "auto",
                "units": "kilometers",
                "directions_options": {"units": "kilometers"}
            })
        );
    }

    #[test]
    fn request_selects_the_costing_of_each_transport_mode() {
        for (transport_mode, expected_costing) in [
            (TransportMode::Car, "auto"),
            (TransportMode::Bicycle, "bicycle"),
            (TransportMode::Pedestrian, "pedestrian"),
        ] {
            let body = build_route_request_body(
                &recorded_waypoints(),
                &RouteOptions::default(),
                transport_mode,
            )
            .expect("the request must encode");
            assert_eq!(body["costing"], expected_costing);
        }
    }

    #[test]
    fn request_omits_costing_options_when_no_option_needs_them() {
        let body = build_route_request_body(
            &recorded_waypoints(),
            &RouteOptions::default(),
            TransportMode::Car,
        )
        .expect("the request must encode");
        assert!(body.get("costing_options").is_none());
    }

    #[test]
    fn shortest_automobile_strategy_sets_the_shortest_factor() {
        let body = build_route_request_body(
            &recorded_waypoints(),
            &RouteOptions {
                route_strategy: Some(RouteStrategy::Shortest),
                ..RouteOptions::default()
            },
            TransportMode::Car,
        )
        .expect("the request must encode");
        assert_eq!(body["costing_options"]["auto"]["shortest"], true);
    }

    #[test]
    fn avoid_flags_map_to_the_automobile_cost_factors() {
        let body = build_route_request_body(
            &recorded_waypoints(),
            &RouteOptions {
                avoid: vec![
                    AvoidType::Tolls,
                    AvoidType::Highways,
                    AvoidType::Ferries,
                    AvoidType::DirtRoads,
                ],
                ..RouteOptions::default()
            },
            TransportMode::Car,
        )
        .expect("the request must encode");
        assert_eq!(
            body["costing_options"],
            json!({"auto": {
                "use_tolls": 0.0,
                "use_highways": 0.0,
                "use_ferry": 0.0,
                "use_tracks": 0.0
            }})
        );
    }

    #[test]
    fn pedestrian_avoidance_omits_factors_the_costing_cannot_express() {
        let body = build_route_request_body(
            &recorded_waypoints(),
            &RouteOptions {
                transport_mode: Some(TransportMode::Pedestrian),
                route_strategy: Some(RouteStrategy::Shortest),
                avoid: vec![
                    AvoidType::Tolls,
                    AvoidType::Highways,
                    AvoidType::Ferries,
                    AvoidType::DirtRoads,
                ],
                ..RouteOptions::default()
            },
            TransportMode::Pedestrian,
        )
        .expect("the request must encode");
        assert_eq!(
            body["costing_options"],
            json!({"pedestrian": {"use_ferry": 0.0}})
        );
    }

    #[test]
    fn recorded_trip_decodes_distance_duration_legs_and_steps() {
        let route = parse_route_payload(fixture("route_sacramento.json"), TransportMode::Car)
            .expect("the trip must parse");
        assert_close(route.distance, 3_200.0);
        assert_close(route.duration, 420.0);
        assert_eq!(route.legs.len(), 1);
        assert_eq!(route.steps.len(), 5);

        let leg = &route.legs[0];
        assert_close(leg.distance, 3_200.0);
        assert_close(leg.duration, 420.0);
        assert_eq!(leg.steps.len(), 5);
        assert_eq!(leg.shape.len(), 12);
        assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
        assert_eq!(
            leg.steps[1].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Right))
        );
        assert_eq!(leg.steps[2].maneuver_type, Some(ManeuverType::Continue));
        assert_eq!(
            leg.steps[3].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Left))
        );
        assert_eq!(leg.steps[4].maneuver_type, Some(ManeuverType::Arrival));
        assert_eq!(
            leg.steps[0].instruction.as_deref(),
            Some("Drive west on N Street.")
        );
        assert_eq!(
            leg.steps[1].instruction.as_deref(),
            Some("Turn right onto 10th Street.")
        );
        assert_eq!(
            leg.steps[1].speech_instruction.as_deref(),
            Some("Turn right onto Tenth Street.")
        );
        assert_eq!(
            leg.steps[4].instruction.as_deref(),
            Some("You have arrived at your destination.")
        );
        assert_eq!(leg.steps[0].shape.len(), 3);
        assert_eq!(leg.steps[3].shape.len(), 4);
        assert_eq!(leg.steps[4].shape.len(), 3);
    }

    #[test]
    fn recorded_trip_bounding_box_encloses_the_decoded_geometry() {
        let route = parse_route_payload(fixture("route_sacramento.json"), TransportMode::Car)
            .expect("the trip must parse");
        let bounding_box = route.bounding_box.expect("the route has a bounding box");
        assert_close(bounding_box.south_west.lat, 38.57652);
        assert_close(bounding_box.south_west.lng, -121.50797);
        assert_close(bounding_box.north_east.lat, 38.57887);
        assert_close(bounding_box.north_east.lng, -121.4936);
    }

    #[test]
    fn a_payload_without_a_trip_is_a_serialization_error() {
        let error = parse_route_payload(
            json!({"error_code": 440, "error": "refused"}),
            TransportMode::Car,
        )
        .unwrap_err();
        assert!(matches!(error, EveryMapError::SerializationError { .. }));
    }

    #[test]
    fn a_trip_without_legs_is_a_provider_error() {
        let error = parse_route_payload(
            json!({"trip": {"summary": {"length": 0.0, "time": 0.0}, "legs": []}}),
            TransportMode::Car,
        )
        .unwrap_err();
        match error {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "INVALID_RESPONSE"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }
}
