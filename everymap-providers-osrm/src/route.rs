//! Route calculation against the OSRM HTTP route service.
//!
//! # Request shape
//!
//! One request is
//! `GET {base_url}/route/v1/{profile}/{longitude},{latitude};…` with
//! `overview=full`, `geometries=geojson`, `steps=true` and, when the caller
//! asks to avoid features, `exclude` set to the requested OSRM classes.
//!
//! # Option mapping
//!
//! - [`TransportMode::Car`] selects the `driving` profile,
//!   [`TransportMode::Bicycle`] the `cycling` profile and
//!   [`TransportMode::Pedestrian`] the `walking` profile. A mode the service
//!   has no profile for falls back to `driving` with a warning.
//! - [`AvoidType::Tolls`] becomes the `toll` class,
//!   [`AvoidType::Highways`] becomes `motorway`,
//!   [`AvoidType::Ferries`] becomes `ferry` and
//!   [`AvoidType::DirtRoads`] becomes `unpaved`, joined in one `exclude`
//!   value. OSRM defines no class for tunnels, so
//!   [`AvoidType::Tunnels`] changes nothing.
//!
//! # Options OSRM cannot express
//!
//! These are documented rather than silently approximated:
//!
//! - [`RouteStrategy`](everymap_core::domains::routing::RouteStrategy) has no
//!   route-service parameter. The deployed profile's
//!   cost model fixes the ranking, so `Fastest` and `Shortest` produce the
//!   same request; [`Router::route_capabilities`] reports that the shortest
//!   strategy is unsupported.
//! - `departure_time`, `arrival_time`, `language` and `alternatives` have no
//!   route-service parameter in the request this provider sends.
//! - OSRM returns no human-readable instruction, so this provider authors
//!   display and speech text from the maneuver and road name, and it does not
//!   map OSRM intersection lane records.
//! - OSRM has no ferry maneuver type. A step whose `mode` is `ferry` is
//!   classified as [`ManeuverType::Ferry`] before the maneuver-type table is
//!   consulted, so a ferry crossing keeps its own instruction instead of
//!   degrading to `Continue`.

use crate::client::{OsrmClient, OsrmProviderConfiguration};
use crate::error_mapping::{interpret_success_body, invalid_response, provider_error_from_body};
use async_trait::async_trait;
use everymap_core::domains::routing::{
    AvoidType, ManeuverType, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult,
    RouteStep, Router, TransportMode, TurnDirection,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpRequest, HttpTransport, Method};
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
use serde_json::Value;
use std::sync::Arc;

/// Path prefix of the route service version the provider targets.
const ROUTE_SERVICE_PATH_PREFIX: &str = "route/v1";

/// OSRM profile segment used for automobile routing.
const DRIVING_PROFILE: &str = "driving";
/// OSRM profile segment used for bicycle routing.
const CYCLING_PROFILE: &str = "cycling";
/// OSRM profile segment used for pedestrian routing.
const WALKING_PROFILE: &str = "walking";

/// OSRM step `mode` value marking a ferry crossing.
const FERRY_STEP_MODE: &str = "ferry";

/// Query parameter requesting a geometry overview.
const OVERVIEW_PARAMETER: &str = "overview";
/// Query value requesting the full route geometry.
const OVERVIEW_FULL: &str = "full";
/// Query parameter selecting the geometry encoding.
const GEOMETRIES_PARAMETER: &str = "geometries";
/// Query value requesting GeoJSON geometries.
const GEOMETRIES_GEOJSON: &str = "geojson";
/// Query parameter requesting turn-by-turn steps.
const STEPS_PARAMETER: &str = "steps";
/// Query value requesting turn-by-turn steps.
const STEPS_TRUE: &str = "true";
/// Query parameter naming the classes the route must avoid.
const EXCLUDE_PARAMETER: &str = "exclude";

/// Separator between the coordinates of successive waypoints.
const COORDINATE_SEPARATOR: &str = ";";
/// Separator between a coordinate's longitude and latitude.
const LONGITUDE_LATITUDE_SEPARATOR: &str = ",";

/// OSRM `exclude` class for toll roads.
const TOLL_EXCLUDE_CLASS: &str = "toll";
/// OSRM `exclude` class for motorways.
const MOTORWAY_EXCLUDE_CLASS: &str = "motorway";
/// OSRM `exclude` class for ferries.
const FERRY_EXCLUDE_CLASS: &str = "ferry";
/// OSRM `exclude` class for unpaved roads.
const UNPAVED_EXCLUDE_CLASS: &str = "unpaved";
/// Separator between OSRM `exclude` classes.
const EXCLUDE_CLASS_SEPARATOR: &str = ",";

/// Header carrying the user agent configured for the deployment.
const USER_AGENT_HEADER_NAME: &str = "user-agent";

/// The fewest waypoints a route can connect.
const MINIMUM_ROUTE_WAYPOINT_COUNT: usize = 2;

/// Exit index used when OSRM reports none for a roundabout. The contract's
/// roundabout index is one-based.
const DEFAULT_ROUNDABOUT_EXIT_INDEX: u8 = 1;

/// Routing through the OSRM HTTP route service.
pub struct OsrmRouter {
    client: Arc<OsrmClient>,
    base_url: String,
    user_agent: String,
}

impl OsrmRouter {
    /// Builds a router for one OSRM deployment on the target's default
    /// transport.
    ///
    /// # Parameters
    /// - `configuration` — deployment base URL and user agent.
    ///
    /// # Returns
    /// A router ready to calculate routes.
    pub fn new(configuration: OsrmProviderConfiguration) -> Self {
        Self::with_client(Arc::new(OsrmClient::keyless()), configuration)
    }

    /// Builds a router that executes requests through an explicit transport.
    ///
    /// # Parameters
    /// - `transport` — the transport that sends each request. Hosts inject
    ///   their platform transport and tests inject a fake.
    /// - `configuration` — deployment base URL and user agent.
    ///
    /// # Returns
    /// A router ready to calculate routes through the supplied transport.
    pub fn with_transport(
        transport: Arc<dyn HttpTransport>,
        configuration: OsrmProviderConfiguration,
    ) -> Self {
        Self::with_client(
            Arc::new(OsrmClient::with_transport(transport)),
            configuration,
        )
    }

    /// Builds a router around a client and validates nothing: the base URL is
    /// joined with each service path at request time.
    fn with_client(client: Arc<OsrmClient>, configuration: OsrmProviderConfiguration) -> Self {
        Self {
            client,
            base_url: configuration.base_url,
            user_agent: configuration.user_agent,
        }
    }

    /// Calculates one route for the ordered waypoints and returns the first
    /// route the service reports.
    ///
    /// # Parameters
    /// - `waypoints` — at least two ordered stops.
    /// - `options` — transport mode, avoidance and strategy choices.
    ///
    /// # Returns
    /// A response with one route whose legs follow the waypoint pairs.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] for fewer than two waypoints.
    /// - [`EveryMapError::ProviderError`] for an OSRM `code` refusal or a
    ///   structurally unusable payload.
    /// - [`EveryMapError::HttpError`] for a non-success status without an OSRM
    ///   envelope.
    /// - [`EveryMapError::SerializationError`] for a success body that is not
    ///   valid JSON.
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
                "a route needs at least {MINIMUM_ROUTE_WAYPOINT_COUNT} waypoints; got {}",
                waypoints.len()
            )));
        }
        let transport_mode = options.transport_mode.unwrap_or(TransportMode::Car);
        let path = route_request_path(transport_mode, waypoints);
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
        let mut request = self
            .client
            .build_request(Method::Get, &url)
            .query(route_query_pairs(options));
        if !self.user_agent.is_empty() {
            request = request.header(USER_AGENT_HEADER_NAME, &self.user_agent);
        }
        let payload = self.send(request).await?;
        let route = parse_route_payload(&payload, transport_mode)?;
        Ok(RouteResponse {
            routes: vec![route],
        })
    }

    /// Sends one request and turns the response into a usable OSRM payload.
    ///
    /// A non-success response that carries an OSRM error envelope becomes a
    /// provider error so the service's own code and message survive; any other
    /// non-success response keeps the shared client's HTTP error.
    async fn send(&self, request: HttpRequest) -> EveryMapResult<Value> {
        match self.client.request(request).await {
            Ok(response) => interpret_success_body(&response.text()),
            Err(EveryMapError::HttpError {
                status,
                message,
                body,
            }) => {
                if let Some(provider_error) = body.as_deref().and_then(provider_error_from_body) {
                    return Err(provider_error);
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
impl Router for OsrmRouter {
    /// Calculates a route between two endpoints.
    ///
    /// # Parameters
    /// - `start` — the origin.
    /// - `end` — the destination.
    /// - `options` — transport mode, avoidance and strategy choices.
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
    /// - `waypoints` — at least two ordered stops; consecutive pairs become
    ///   the returned legs.
    /// - `options` — transport mode, avoidance and strategy choices.
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

    /// OSRM's route service selects a profile by mode and has no
    /// shortest-path switch, so the strategy choice cannot be honoured.
    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Bicycle,
                TransportMode::Pedestrian,
            ],
            supports_shortest_route_strategy: false,
        }
    }
}

/// Builds the route-service path, including the profile and the ordered
/// `longitude,latitude` waypoint list.
fn route_request_path(transport_mode: TransportMode, waypoints: &[Coordinate]) -> String {
    let coordinates = waypoints
        .iter()
        .map(|waypoint| {
            format!(
                "{}{LONGITUDE_LATITUDE_SEPARATOR}{}",
                waypoint.lng, waypoint.lat
            )
        })
        .collect::<Vec<String>>()
        .join(COORDINATE_SEPARATOR);
    format!(
        "{ROUTE_SERVICE_PATH_PREFIX}/{}/{}",
        profile_path_segment(transport_mode),
        coordinates
    )
}

/// Maps the contract's transport mode onto OSRM's profile path segment.
fn profile_path_segment(transport_mode: TransportMode) -> &'static str {
    match transport_mode {
        TransportMode::Car => DRIVING_PROFILE,
        TransportMode::Bicycle => CYCLING_PROFILE,
        TransportMode::Pedestrian => WALKING_PROFILE,
        unsupported => {
            log::warn!(
                "OSRM routing does not support transport mode {unsupported:?}, falling back to the driving profile"
            );
            DRIVING_PROFILE
        }
    }
}

/// Builds the route-service query pairs, including avoidance classes when the
/// caller asked for any.
fn route_query_pairs(options: &RouteOptions) -> Vec<(String, String)> {
    let mut query_pairs = vec![
        (OVERVIEW_PARAMETER.to_string(), OVERVIEW_FULL.to_string()),
        (
            GEOMETRIES_PARAMETER.to_string(),
            GEOMETRIES_GEOJSON.to_string(),
        ),
        (STEPS_PARAMETER.to_string(), STEPS_TRUE.to_string()),
    ];
    let exclude_classes = exclude_classes_text(&options.avoid);
    if !exclude_classes.is_empty() {
        query_pairs.push((EXCLUDE_PARAMETER.to_string(), exclude_classes));
    }
    query_pairs
}

/// Joins the requested avoidance classes into one OSRM `exclude` value.
fn exclude_classes_text(avoid: &[AvoidType]) -> String {
    let mut classes: Vec<&str> = Vec::new();
    for avoid_type in avoid {
        let exclude_class = match avoid_type {
            AvoidType::Tolls => Some(TOLL_EXCLUDE_CLASS),
            AvoidType::Highways => Some(MOTORWAY_EXCLUDE_CLASS),
            AvoidType::Ferries => Some(FERRY_EXCLUDE_CLASS),
            AvoidType::DirtRoads => Some(UNPAVED_EXCLUDE_CLASS),
            AvoidType::Tunnels => {
                log::warn!(
                    "OSRM has no exclude class for tunnels; the request does not avoid them"
                );
                None
            }
        };
        if let Some(exclude_class) = exclude_class {
            if !classes.contains(&exclude_class) {
                classes.push(exclude_class);
            }
        }
    }
    classes.join(EXCLUDE_CLASS_SEPARATOR)
}

/// Parses an OSRM route payload into a [`RouteResult`].
///
/// # Parameters
/// - `payload` — a decoded OSRM route response whose `code` is `Ok`.
/// - `transport_mode` — the mode the request asked for, echoed on the result.
///
/// # Returns
/// The first route's total distance and duration, its full GeoJSON geometry
/// and derived bounding box, and one leg per consecutive waypoint pair. The
/// flat step list is the concatenation of every leg's steps.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] with the `INVALID_RESPONSE` code
/// when the payload lacks a route, its numeric totals, its GeoJSON geometry,
/// its legs or a leg's steps, or when a coordinate is outside the valid
/// ranges.
fn parse_route_payload(
    payload: &Value,
    transport_mode: TransportMode,
) -> EveryMapResult<RouteResult> {
    let routes = payload
        .get("routes")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("OSRM route payload carries no routes array"))?;
    let route = routes
        .first()
        .ok_or_else(|| invalid_response("OSRM route payload carries no routes"))?;

    let distance = required_number(route, "distance", "route")?;
    let duration = required_number(route, "duration", "route")?;

    let geometry = route
        .get("geometry")
        .ok_or_else(|| invalid_response("OSRM route payload carries no geometry"))?;
    let route_shape = parse_geojson_line_string(geometry, "route geometry")?;
    let bounding_box = bounding_box_from_coordinates(&route_shape)?;

    let leg_values = route
        .get("legs")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("OSRM route payload carries no legs array"))?;
    if leg_values.is_empty() {
        return Err(invalid_response("OSRM route payload carries no legs"));
    }
    let legs = leg_values
        .iter()
        .map(parse_route_leg)
        .collect::<EveryMapResult<Vec<RouteLeg>>>()?;
    let steps = legs
        .iter()
        .flat_map(|leg| leg.steps.iter().cloned())
        .collect();

    Ok(RouteResult {
        distance,
        duration,
        geometry: Polyline::new(route_shape),
        transport_mode: Some(transport_mode),
        steps,
        bounding_box: Some(bounding_box),
        raw: None,
        legs,
    })
}

/// Parses one leg: totals, its ordered steps and the leg geometry they cover.
fn parse_route_leg(leg: &Value) -> EveryMapResult<RouteLeg> {
    let distance = required_number(leg, "distance", "route leg")?;
    let duration = required_number(leg, "duration", "route leg")?;
    let step_values = leg
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("OSRM route leg carries no steps array"))?;
    let steps = step_values
        .iter()
        .map(parse_route_step)
        .collect::<EveryMapResult<Vec<RouteStep>>>()?;
    let shape = shape_from_steps(&steps);
    Ok(RouteLeg {
        distance,
        duration,
        steps,
        shape,
    })
}

/// Parses one step: totals, maneuver classification, authored instruction
/// text and the step's GeoJSON shape.
fn parse_route_step(step: &Value) -> EveryMapResult<RouteStep> {
    let distance = required_number(step, "distance", "route step")?;
    let duration = required_number(step, "duration", "route step")?;
    let maneuver = step
        .get("maneuver")
        .ok_or_else(|| invalid_response("OSRM route step carries no maneuver"))?;
    let maneuver_type_text = maneuver
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_response("OSRM route step maneuver carries no type"))?;
    let modifier = maneuver.get("modifier").and_then(Value::as_str);
    let exit_index = maneuver
        .get("exit")
        .and_then(Value::as_u64)
        .map(|exit| exit.min(u64::from(u8::MAX)) as u8);
    // OSRM marks a ferry crossing on the step's `mode`, not on the maneuver
    // type, whose vocabulary has no ferry entry. The step `mode` takes
    // precedence so an entering, crossing or leaving ferry step is classified
    // as a ferry maneuver.
    let maneuver_type = if step.get("mode").and_then(Value::as_str) == Some(FERRY_STEP_MODE) {
        ManeuverType::Ferry
    } else {
        map_osrm_maneuver(maneuver_type_text, modifier, exit_index)
    };

    let road_name = step.get("name").and_then(Value::as_str).unwrap_or_default();
    let instruction = instruction_text(&maneuver_type, road_name);
    let shape = match step.get("geometry") {
        Some(geometry) => parse_geojson_line_string(geometry, "step geometry")?,
        None => Vec::new(),
    };
    let start_coordinate = maneuver
        .get("location")
        .and_then(parse_position)
        .or_else(|| shape.first().copied());
    let end_coordinate = shape.last().copied();

    Ok(RouteStep {
        instruction: Some(instruction.clone()),
        distance: Some(distance),
        duration: Some(duration),
        start_coordinate,
        end_coordinate,
        maneuver_type: Some(maneuver_type),
        speech_instruction: Some(instruction),
        lane_guidances: Vec::new(),
        shape,
    })
}

/// Concatenates the step shapes into the leg geometry, dropping the duplicate
/// point where one step ends and the next begins.
fn shape_from_steps(steps: &[RouteStep]) -> Vec<Coordinate> {
    let mut shape = Vec::new();
    for step in steps {
        for point in &step.shape {
            if shape.last() != Some(point) {
                shape.push(*point);
            }
        }
    }
    shape
}

/// Reads a required numeric field from a payload object.
fn required_number(container: &Value, field_name: &str, context: &str) -> EveryMapResult<f64> {
    container
        .get(field_name)
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_response(format!("OSRM {context} carries no numeric {field_name}")))
}

/// Parses one GeoJSON position (`[longitude, latitude]`) into a coordinate.
///
/// Returns `None` for a position that is not a numeric pair or lies outside
/// the valid ranges; positions that annotate a step are optional metadata.
fn parse_position(position: &Value) -> Option<Coordinate> {
    let values = position.as_array()?;
    let longitude = values.first()?.as_f64()?;
    let latitude = values.get(1)?.as_f64()?;
    Coordinate::new(latitude, longitude).ok()
}

/// Parses a GeoJSON `LineString` geometry into coordinates.
///
/// GeoJSON positions are `[longitude, latitude]`; the contract's coordinate
/// exposes the same pair in named fields.
fn parse_geojson_line_string(geometry: &Value, context: &str) -> EveryMapResult<Vec<Coordinate>> {
    if let Some(geometry_type) = geometry.get("type").and_then(Value::as_str) {
        if geometry_type != "LineString" {
            return Err(invalid_response(format!(
                "OSRM {context} is {geometry_type}, not LineString"
            )));
        }
    }
    let coordinate_values = geometry
        .get("coordinates")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response(format!("OSRM {context} carries no coordinates array")))?;
    let mut coordinates = Vec::with_capacity(coordinate_values.len());
    for coordinate_value in coordinate_values {
        let position = coordinate_value.as_array().ok_or_else(|| {
            invalid_response(format!(
                "OSRM {context} has a position that is not an array"
            ))
        })?;
        if position.len() < 2 {
            return Err(invalid_response(format!(
                "OSRM {context} has a position with fewer than two elements"
            )));
        }
        let longitude = position[0].as_f64().ok_or_else(|| {
            invalid_response(format!("OSRM {context} has a non-numeric longitude"))
        })?;
        let latitude = position[1].as_f64().ok_or_else(|| {
            invalid_response(format!("OSRM {context} has a non-numeric latitude"))
        })?;
        coordinates.push(Coordinate::new(latitude, longitude).map_err(|error| {
            invalid_response(format!("OSRM {context} has an invalid coordinate: {error}"))
        })?);
    }
    Ok(coordinates)
}

/// Derives the route bounding box from its full geometry.
fn bounding_box_from_coordinates(coordinates: &[Coordinate]) -> EveryMapResult<BoundingBox> {
    let first = coordinates
        .first()
        .ok_or_else(|| invalid_response("OSRM route geometry is empty"))?;
    let mut minimum_latitude = first.lat;
    let mut maximum_latitude = first.lat;
    let mut minimum_longitude = first.lng;
    let mut maximum_longitude = first.lng;
    for coordinate in coordinates {
        minimum_latitude = minimum_latitude.min(coordinate.lat);
        maximum_latitude = maximum_latitude.max(coordinate.lat);
        minimum_longitude = minimum_longitude.min(coordinate.lng);
        maximum_longitude = maximum_longitude.max(coordinate.lng);
    }
    let south_west = Coordinate::new(minimum_latitude, minimum_longitude).map_err(|error| {
        invalid_response(format!(
            "OSRM route geometry bounding box is invalid: {error}"
        ))
    })?;
    let north_east = Coordinate::new(maximum_latitude, maximum_longitude).map_err(|error| {
        invalid_response(format!(
            "OSRM route geometry bounding box is invalid: {error}"
        ))
    })?;
    Ok(BoundingBox::new(north_east, south_west))
}

/// Maps OSRM's documented maneuver vocabulary onto [`ManeuverType`].
///
/// | OSRM `maneuver.type` | Contract maneuver |
/// |---|---|
/// | `depart` | [`ManeuverType::Departure`] |
/// | `arrive` | [`ManeuverType::Arrival`] |
/// | `turn`, `end of road` | [`ManeuverType::Turn`] from `modifier`; [`ManeuverType::Continue`] when the modifier is missing or unknown |
/// | `continue` | [`ManeuverType::Turn`] from `modifier`; `Turn(Straight)` when the modifier is missing or unknown |
/// | `new name`, `use lane`, `notification` | [`ManeuverType::Continue`] |
/// | `merge` | [`ManeuverType::Merge`] |
/// | `on ramp` | [`ManeuverType::OnRamp`] |
/// | `off ramp` | [`ManeuverType::OffRamp`] |
/// | `fork` | [`ManeuverType::Fork`] |
/// | `roundabout`, `rotary`, `roundabout turn`, `exit roundabout`, `exit rotary` | [`ManeuverType::Roundabout`] with the one-based `exit` index |
/// | anything else | [`ManeuverType::Continue`] |
///
/// Ferry crossings are not part of this table: OSRM marks them on the step's
/// `mode` field, and a step whose mode is `ferry` maps to
/// [`ManeuverType::Ferry`] before this function is consulted.
///
/// The modifier vocabulary is `uturn` (or `u-turn`), `sharp left`, `left`,
/// `slight left`, `straight`, `slight right`, `right` and `sharp right`. A
/// `turn` or `end of road` maneuver with no recognized modifier maps to
/// [`ManeuverType::Continue`] rather than guessing a turn direction. A
/// roundabout without a positive `exit` index maps to exit `1`, keeping every
/// maneuver's stable label valid.
///
/// # Parameters
/// - `maneuver_type` — OSRM's `maneuver.type` string.
/// - `modifier` — OSRM's optional `maneuver.modifier` string.
/// - `exit_index` — OSRM's optional one-based roundabout `exit` index.
///
/// # Returns
/// The contract maneuver for the OSRM classification. Unknown types use the
/// documented [`ManeuverType::Continue`] fallback.
pub fn map_osrm_maneuver(
    maneuver_type: &str,
    modifier: Option<&str>,
    exit_index: Option<u8>,
) -> ManeuverType {
    match maneuver_type {
        "depart" => ManeuverType::Departure,
        "arrive" => ManeuverType::Arrival,
        "turn" | "end of road" => turn_from_modifier(modifier).unwrap_or(ManeuverType::Continue),
        "continue" => {
            turn_from_modifier(modifier).unwrap_or(ManeuverType::Turn(TurnDirection::Straight))
        }
        "new name" | "use lane" | "notification" => ManeuverType::Continue,
        "merge" => ManeuverType::Merge,
        "on ramp" => ManeuverType::OnRamp,
        "off ramp" => ManeuverType::OffRamp,
        "fork" => ManeuverType::Fork,
        "roundabout" | "rotary" | "roundabout turn" | "exit roundabout" | "exit rotary" => {
            ManeuverType::Roundabout(
                exit_index
                    .filter(|index| *index >= DEFAULT_ROUNDABOUT_EXIT_INDEX)
                    .unwrap_or(DEFAULT_ROUNDABOUT_EXIT_INDEX),
            )
        }
        _ => ManeuverType::Continue,
    }
}

/// Builds the turn maneuver a recognized OSRM modifier names; `None` for an
/// absent or unrecognized modifier.
fn turn_from_modifier(modifier: Option<&str>) -> Option<ManeuverType> {
    let direction = match modifier? {
        "straight" => TurnDirection::Straight,
        "slight left" => TurnDirection::SlightLeft,
        "left" => TurnDirection::Left,
        "sharp left" => TurnDirection::SharpLeft,
        "slight right" => TurnDirection::SlightRight,
        "right" => TurnDirection::Right,
        "sharp right" => TurnDirection::SharpRight,
        "uturn" | "u-turn" => TurnDirection::UTurn,
        _ => return None,
    };
    Some(ManeuverType::Turn(direction))
}

/// Authors the display instruction for one maneuver.
///
/// OSRM returns maneuver classifications rather than prose, so the text is
/// generated from the maneuver and the road name OSRM reports for the step.
/// The speech string carries the same text; there is no separate spoken form
/// in the provider payload.
fn instruction_text(maneuver_type: &ManeuverType, road_name: &str) -> String {
    let onto = if road_name.is_empty() {
        String::new()
    } else {
        format!(" onto {road_name}")
    };
    match maneuver_type {
        ManeuverType::Departure => {
            if road_name.is_empty() {
                "Start navigation".to_string()
            } else {
                format!("Start on {road_name}")
            }
        }
        ManeuverType::Arrival => "Arrive at your destination".to_string(),
        ManeuverType::Continue => format!("Continue{onto}"),
        ManeuverType::Turn(direction) => {
            let action = match direction {
                TurnDirection::Straight => "Continue straight",
                TurnDirection::SlightLeft => "Turn slight left",
                TurnDirection::Left => "Turn left",
                TurnDirection::SharpLeft => "Turn sharp left",
                TurnDirection::SlightRight => "Turn slight right",
                TurnDirection::Right => "Turn right",
                TurnDirection::SharpRight => "Turn sharp right",
                TurnDirection::UTurn => "Make a U-turn",
                TurnDirection::LeavingRoad => "Leave the current road",
            };
            format!("{action}{onto}")
        }
        ManeuverType::Merge => format!("Merge{onto}"),
        ManeuverType::OnRamp => format!("Take the ramp{onto}"),
        ManeuverType::OffRamp => format!("Take the exit{onto}"),
        ManeuverType::Fork => format!("Take the fork{onto}"),
        ManeuverType::Roundabout(exit_index) => {
            format!("Take exit {exit_index} at the roundabout")
        }
        ManeuverType::Ferry => "Take the ferry".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        exclude_classes_text, instruction_text, map_osrm_maneuver, parse_route_payload,
        profile_path_segment, route_query_pairs, route_request_path,
    };
    use everymap_core::domains::routing::{
        AvoidType, ManeuverType, RouteOptions, RouteStrategy, TransportMode, TurnDirection,
    };
    use everymap_core::error::EveryMapError;
    use everymap_core::types::Coordinate;
    use serde_json::Value;

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(fixture_path).unwrap()).unwrap()
    }

    fn coordinate(latitude: f64, longitude: f64) -> Coordinate {
        Coordinate::new(latitude, longitude).unwrap()
    }

    fn sacramento_waypoints() -> Vec<Coordinate> {
        vec![
            coordinate(38.57657, -121.4936),
            coordinate(38.5781, -121.50555),
        ]
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn sacramento_payload_decodes_totals_legs_steps_and_bounds() {
        let route =
            parse_route_payload(&fixture("sacramento_route.json"), TransportMode::Car).unwrap();
        assert_close(route.distance, 1302.3);
        assert_close(route.duration, 157.1);
        assert_eq!(route.geometry.points.len(), 9);
        assert_eq!(route.legs.len(), 1);
        assert_eq!(route.steps.len(), 4);
        assert_eq!(route.transport_mode, Some(TransportMode::Car));

        let leg = &route.legs[0];
        assert_close(leg.distance, 1302.3);
        assert_close(leg.duration, 157.1);
        assert_eq!(leg.steps.len(), 4);
        assert!(!leg.shape.is_empty(), "the leg carries the step geometry");

        assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
        assert_eq!(
            leg.steps[1].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Left))
        );
        assert_eq!(
            leg.steps[2].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Right))
        );
        assert_eq!(leg.steps[3].maneuver_type, Some(ManeuverType::Arrival));

        assert_eq!(
            leg.steps[0].instruction.as_deref(),
            Some("Start on 10th Street")
        );
        assert_eq!(
            leg.steps[1].instruction.as_deref(),
            Some("Turn left onto L Street")
        );
        assert_eq!(
            leg.steps[2].instruction.as_deref(),
            Some("Turn right onto Front Street")
        );
        for step in &leg.steps {
            assert!(!step.shape.is_empty(), "every recorded step has a shape");
            assert_eq!(step.speech_instruction, step.instruction);
            assert!(step.lane_guidances.is_empty());
        }

        let bounding_box = route.bounding_box.expect("the route has a bounding box");
        assert_close(bounding_box.south_west.lat, 38.57657);
        assert_close(bounding_box.south_west.lng, -121.50555);
        assert_close(bounding_box.north_east.lat, 38.5781);
        assert_close(bounding_box.north_east.lng, -121.4936);
    }

    #[test]
    fn ferry_steps_classify_as_ferry_from_their_mode() {
        let route = parse_route_payload(
            &fixture("san_francisco_oakland_ferry_route.json"),
            TransportMode::Car,
        )
        .unwrap();
        assert_close(route.distance, 10978.4);
        assert_close(route.duration, 1524.6);
        let steps = &route.legs[0].steps;
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].maneuver_type, Some(ManeuverType::Departure));
        assert_eq!(steps[2].maneuver_type, Some(ManeuverType::Arrival));

        let ferry_step = steps
            .iter()
            .find(|step| step.maneuver_type == Some(ManeuverType::Ferry))
            .expect("the crossing step must classify as a ferry maneuver");
        assert_eq!(ferry_step.instruction.as_deref(), Some("Take the ferry"));
        assert_eq!(
            ferry_step.speech_instruction.as_deref(),
            Some("Take the ferry")
        );
    }

    #[test]
    fn no_route_payload_reports_the_service_code() {
        let payload = fixture("no_route.json");
        let error = crate::error_mapping::provider_error_from_payload(&payload)
            .expect("a NoRoute envelope must surface");
        match error {
            EveryMapError::ProviderError {
                provider,
                code,
                message,
            } => {
                assert_eq!(provider, "osrm");
                assert_eq!(code, "NoRoute");
                assert_eq!(message, "Impossible route between points");
            }
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn maneuver_mapping_table_covers_every_handled_osrm_type() {
        let table = [
            ("depart", None, None, ManeuverType::Departure),
            ("arrive", None, None, ManeuverType::Arrival),
            (
                "turn",
                Some("left"),
                None,
                ManeuverType::Turn(TurnDirection::Left),
            ),
            (
                "end of road",
                Some("right"),
                None,
                ManeuverType::Turn(TurnDirection::Right),
            ),
            (
                "continue",
                None,
                None,
                ManeuverType::Turn(TurnDirection::Straight),
            ),
            ("new name", None, None, ManeuverType::Continue),
            ("use lane", None, None, ManeuverType::Continue),
            ("notification", None, None, ManeuverType::Continue),
            ("merge", None, None, ManeuverType::Merge),
            ("on ramp", None, None, ManeuverType::OnRamp),
            ("off ramp", None, None, ManeuverType::OffRamp),
            ("fork", None, None, ManeuverType::Fork),
            ("roundabout", None, Some(3), ManeuverType::Roundabout(3)),
            ("rotary", None, Some(2), ManeuverType::Roundabout(2)),
            (
                "roundabout turn",
                None,
                Some(1),
                ManeuverType::Roundabout(1),
            ),
            (
                "exit roundabout",
                None,
                Some(4),
                ManeuverType::Roundabout(4),
            ),
            ("exit rotary", None, Some(5), ManeuverType::Roundabout(5)),
        ];
        for (maneuver_type, modifier, exit_index, expected) in table {
            assert_eq!(
                map_osrm_maneuver(maneuver_type, modifier, exit_index),
                expected,
                "OSRM type {maneuver_type:?} mapped incorrectly"
            );
        }
    }

    #[test]
    fn unknown_maneuver_types_and_modifiers_use_the_documented_fallbacks() {
        assert_eq!(
            map_osrm_maneuver("provider extension", None, None),
            ManeuverType::Continue
        );
        assert_eq!(
            map_osrm_maneuver("turn", Some("sideways"), None),
            ManeuverType::Continue,
            "an unrecognized turn modifier must not be guessed as a direction"
        );
        assert_eq!(
            map_osrm_maneuver("turn", None, None),
            ManeuverType::Continue
        );
        assert_eq!(
            map_osrm_maneuver("roundabout", None, None),
            ManeuverType::Roundabout(1)
        );
        assert_eq!(
            map_osrm_maneuver("roundabout", None, Some(0)),
            ManeuverType::Roundabout(1)
        );
    }

    #[test]
    fn turn_modifiers_map_to_every_contract_direction() {
        let table = [
            ("straight", TurnDirection::Straight),
            ("slight left", TurnDirection::SlightLeft),
            ("left", TurnDirection::Left),
            ("sharp left", TurnDirection::SharpLeft),
            ("slight right", TurnDirection::SlightRight),
            ("right", TurnDirection::Right),
            ("sharp right", TurnDirection::SharpRight),
            ("uturn", TurnDirection::UTurn),
            ("u-turn", TurnDirection::UTurn),
        ];
        for (modifier, expected) in table {
            assert_eq!(
                map_osrm_maneuver("turn", Some(modifier), None),
                ManeuverType::Turn(expected),
                "modifier {modifier:?} mapped incorrectly"
            );
        }
    }

    #[test]
    fn profile_segments_follow_the_transport_mode() {
        assert_eq!(profile_path_segment(TransportMode::Car), "driving");
        assert_eq!(profile_path_segment(TransportMode::Bicycle), "cycling");
        assert_eq!(profile_path_segment(TransportMode::Pedestrian), "walking");
        assert_eq!(profile_path_segment(TransportMode::Truck), "driving");
    }

    #[test]
    fn avoidance_options_become_exclude_classes() {
        assert!(exclude_classes_text(&[]).is_empty());
        assert_eq!(
            exclude_classes_text(&[
                AvoidType::Tolls,
                AvoidType::Highways,
                AvoidType::Ferries,
                AvoidType::DirtRoads,
            ]),
            "toll,motorway,ferry,unpaved"
        );
        assert_eq!(
            exclude_classes_text(&[AvoidType::Tunnels]),
            "",
            "OSRM has no tunnel class"
        );
        assert_eq!(
            exclude_classes_text(&[AvoidType::Tolls, AvoidType::Tolls]),
            "toll",
            "a repeated class is sent once"
        );
    }

    #[test]
    fn query_pairs_never_add_an_empty_exclude_value() {
        let names: Vec<String> = route_query_pairs(&RouteOptions::default())
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert_eq!(names, vec!["overview", "geometries", "steps"]);
    }

    #[test]
    fn strategy_does_not_change_the_request() {
        let fastest = RouteOptions {
            route_strategy: Some(RouteStrategy::Fastest),
            ..RouteOptions::default()
        };
        let shortest = RouteOptions {
            route_strategy: Some(RouteStrategy::Shortest),
            ..RouteOptions::default()
        };
        assert_eq!(route_query_pairs(&fastest), route_query_pairs(&shortest));
    }

    #[test]
    fn waypoint_order_and_profile_are_in_the_path() {
        assert_eq!(
            route_request_path(TransportMode::Car, &sacramento_waypoints()),
            "route/v1/driving/-121.4936,38.57657;-121.50555,38.5781"
        );
        assert_eq!(
            route_request_path(TransportMode::Pedestrian, &sacramento_waypoints()),
            "route/v1/walking/-121.4936,38.57657;-121.50555,38.5781"
        );
    }

    #[test]
    fn instruction_text_is_authored_from_the_maneuver_and_road_name() {
        assert_eq!(
            instruction_text(&ManeuverType::Departure, ""),
            "Start navigation"
        );
        assert_eq!(
            instruction_text(&ManeuverType::Turn(TurnDirection::UTurn), "Capitol Mall"),
            "Make a U-turn onto Capitol Mall"
        );
        assert_eq!(instruction_text(&ManeuverType::Merge, ""), "Merge");
        assert_eq!(
            instruction_text(&ManeuverType::Roundabout(2), "L Street"),
            "Take exit 2 at the roundabout"
        );
    }

    #[test]
    fn payload_without_routes_is_a_provider_error() {
        let error = parse_route_payload(&serde_json::json!({"code": "Ok"}), TransportMode::Car)
            .unwrap_err();
        match error {
            EveryMapError::ProviderError { code, .. } => assert_eq!(code, "INVALID_RESPONSE"),
            other => panic!("expected a provider error, got {other:?}"),
        }
    }

    #[test]
    fn empty_routes_array_is_a_provider_error() {
        let payload = serde_json::json!({"code": "Ok", "routes": []});
        assert!(parse_route_payload(&payload, TransportMode::Car).is_err());
    }
}
