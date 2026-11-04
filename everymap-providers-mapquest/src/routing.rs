//! Routing through the MapQuest Directions API v2 route endpoint.
//!
//! The router sends the documented `from`/`to` coordinates, requests the raw
//! full shape and text narratives, and projects the route into the core's
//! navigation-grade
//! [`RouteResponse`]: one
//! [`RouteResult`] with a
//! polyline geometry, a bounding box, one
//! [`RouteLeg`] per consecutive
//! waypoint pair and one
//! [`RouteStep`] per maneuver.
//!
//! # Request shape
//!
//! One request is `GET {base_url}/directions/v2/route` with:
//!
//! - `from` and one `to` per remaining waypoint, in request order;
//! - `unit=k` (the core contract carries meters, so the request asks for
//!   kilometres and converts the answer);
//! - `routeType` — `fastest` or `shortest` for automobile routing and the
//!   fixed `bicycle` or `pedestrian` value for those modes;
//! - `shapeFormat=raw`, `fullShape=true`, `narrativeType=text`;
//! - `avoids` when the caller asks to avoid tolls, highways, ferries or dirt
//!   roads.
//!
//! # Options the service cannot express
//!
//! The Directions API v2 request this provider sends has no parameter for
//! `alternatives`, `departure_time`, `arrival_time`, `language` or
//! `provider_extra`, so those options are reported as unsupported through the
//! log rather than silently approximated. The service also defines no avoid
//! class for tunnels, so [`AvoidType::Tunnels`] changes nothing.
//!
//! # Step text
//!
//! A MapQuest maneuver's narrative describes the action taken at that
//! maneuver's point, while the step's shape runs from one maneuver point to
//! the next. The provider therefore gives each step the narrative of the
//! maneuver at the step's end (the following maneuver), which is what a
//! turn-by-turn banner shows while the step is traversed. The last step keeps
//! its own arrival narrative. This is the same end-anchored projection the
//! MapQuest reference implementation uses.

use crate::client::MapQuestClient;
use crate::error_mapping::invalid_response;
use crate::json_coordinates::coordinate_to_location_query_text;
use async_trait::async_trait;
use everymap_core::domains::routing::{
    AvoidType, ManeuverType, RouteCapabilities, RouteLeg, RouteOptions, RouteResponse, RouteResult,
    RouteStep, RouteStrategy, Router, TransportMode, TurnDirection,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::{BoundingBox, Coordinate, Polyline};
use serde_json::Value;
use std::sync::Arc;

/// Service path of the route endpoint.
const ROUTE_PATH: &str = "directions/v2/route";

/// Query parameter carrying the origin coordinate.
const FROM_PARAMETER: &str = "from";

/// Query parameter carrying one destination or intermediate coordinate.
const TO_PARAMETER: &str = "to";

/// Query parameter selecting the distance unit.
const UNIT_PARAMETER: &str = "unit";

/// Unit value requesting kilometres; the projection converts them to meters.
const METRIC_UNIT_VALUE: &str = "k";

/// Query parameter selecting the route cost model.
const ROUTE_TYPE_PARAMETER: &str = "routeType";

/// Route type minimizing travel time.
const FASTEST_ROUTE_TYPE: &str = "fastest";

/// Route type minimizing traveled distance.
const SHORTEST_ROUTE_TYPE: &str = "shortest";

/// Route type for bicycle routing.
const BICYCLE_ROUTE_TYPE: &str = "bicycle";

/// Route type for pedestrian routing.
const PEDESTRIAN_ROUTE_TYPE: &str = "pedestrian";

/// Query parameter selecting the shape encoding.
const SHAPE_FORMAT_PARAMETER: &str = "shapeFormat";

/// Shape encoding value requesting the raw coordinate array.
const RAW_SHAPE_FORMAT: &str = "raw";

/// Query parameter requesting the complete route shape.
const FULL_SHAPE_PARAMETER: &str = "fullShape";

/// Boolean query value used by the request.
const TRUE_VALUE: &str = "true";

/// Query parameter selecting the narrative form.
const NARRATIVE_TYPE_PARAMETER: &str = "narrativeType";

/// Narrative value requesting plain text.
const TEXT_NARRATIVE_TYPE: &str = "text";

/// Query parameter naming the features to avoid.
const AVOIDS_PARAMETER: &str = "avoids";

/// Documented avoid feature for toll roads.
const TOLL_ROAD_AVOID_FEATURE: &str = "Toll Road";

/// Documented avoid feature for limited-access highways.
const LIMITED_ACCESS_AVOID_FEATURE: &str = "Limited Access";

/// Documented avoid feature for ferries.
const FERRY_AVOID_FEATURE: &str = "Ferry";

/// Documented avoid feature for unpaved roads.
const UNPAVED_AVOID_FEATURE: &str = "Unpaved";

/// Separator between avoid features.
const AVOID_FEATURE_SEPARATOR: &str = ",";

/// Meters in one kilometre.
const METERS_PER_KILOMETER: f64 = 1000.0;

/// The fewest waypoints a route can connect.
const MINIMUM_ROUTE_WAYPOINT_COUNT: usize = 2;

/// Exit index assumed when a roundabout narrative carries no ordinal.
const DEFAULT_ROUNDABOUT_EXIT_INDEX: u8 = 1;

/// Routing through the MapQuest Directions API v2 route endpoint.
///
/// Construct one with [`Self::new`] around a shared [`MapQuestClient`]. The
/// router reports its supported choices through
/// [`Router::route_capabilities`]: automobile, bicycle and pedestrian modes,
/// with both the fastest and shortest strategies.
pub struct MapQuestRouter {
    client: Arc<MapQuestClient>,
}

impl MapQuestRouter {
    /// Builds a router around a shared MapQuest client.
    ///
    /// # Parameters
    /// - `client` — the client every request goes through.
    ///
    /// # Returns
    /// A router ready for two-endpoint and multi-waypoint requests.
    pub fn new(client: Arc<MapQuestClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Router for MapQuestRouter {
    /// Calculates a route between two endpoints.
    ///
    /// # Parameters
    /// - `start` — the origin waypoint.
    /// - `end` — the destination waypoint.
    /// - `options` — transport mode, strategy, avoidance and other choices.
    ///
    /// # Returns
    /// A response with the single route the service returned.
    ///
    /// # Errors
    /// Returns the same failures as [`Self::calculate_route_with_waypoints`].
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        self.calculate_route_with_waypoints(&[*start, *end], options)
            .await
    }

    /// Calculates a route through ordered waypoints.
    ///
    /// # Parameters
    /// - `waypoints` — the ordered stops, origin first and destination last.
    ///   At least two are required; every waypoint after the first becomes one
    ///   `to` query value, so the service returns one leg per consecutive
    ///   pair.
    /// - `options` — transport mode, strategy, avoidance and other choices.
    ///
    /// # Returns
    /// A response with one route carrying its legs, steps, geometry and
    /// bounding box.
    ///
    /// # Errors
    /// - [`EveryMapError::ValidationError`] when fewer than two waypoints are
    ///   supplied.
    /// - The shared client's HTTP, rate-limit or transport error for an
    ///   unsuccessful response.
    /// - [`EveryMapError::SerializationError`] when the body is not JSON.
    /// - [`EveryMapError::ProviderError`] when the service reports a non-zero
    ///   `info.statuscode` or the route payload lacks a required field.
    async fn calculate_route_with_waypoints(
        &self,
        waypoints: &[Coordinate],
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        if waypoints.len() < MINIMUM_ROUTE_WAYPOINT_COUNT {
            return Err(EveryMapError::ValidationError(
                "routing requires at least an origin and a destination".to_string(),
            ));
        }
        let transport_mode = effective_transport_mode(options.transport_mode);
        let mut query_parameters = vec![
            (
                FROM_PARAMETER.to_string(),
                coordinate_to_location_query_text(&waypoints[0]),
            ),
            (UNIT_PARAMETER.to_string(), METRIC_UNIT_VALUE.to_string()),
            (
                ROUTE_TYPE_PARAMETER.to_string(),
                route_type_query_text(transport_mode, options.route_strategy).to_string(),
            ),
            (
                SHAPE_FORMAT_PARAMETER.to_string(),
                RAW_SHAPE_FORMAT.to_string(),
            ),
            (FULL_SHAPE_PARAMETER.to_string(), TRUE_VALUE.to_string()),
            (
                NARRATIVE_TYPE_PARAMETER.to_string(),
                TEXT_NARRATIVE_TYPE.to_string(),
            ),
        ];
        if let Some(avoid_features) = avoid_query_text(&options.avoid) {
            query_parameters.push((AVOIDS_PARAMETER.to_string(), avoid_features));
        }
        for waypoint in &waypoints[1..] {
            query_parameters.push((
                TO_PARAMETER.to_string(),
                coordinate_to_location_query_text(waypoint),
            ));
        }
        report_unsupported_options(options);
        let payload = self.client.get_json(ROUTE_PATH, &query_parameters).await?;
        parse_route_payload(&payload, transport_mode)
    }

    /// Reports the choices this service honours.
    ///
    /// # Returns
    /// Automobile, bicycle and pedestrian transport modes, and support for the
    /// shortest route strategy. Other modes have no documented route type on
    /// this endpoint.
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

/// Projects a requested transport mode onto one this service routes for.
///
/// Automobile, bicycle and pedestrian are honoured. A mode the service has no
/// route type for is routed as an automobile with a log warning, matching the
/// other routing providers in the workspace.
pub(crate) fn effective_transport_mode(requested_mode: Option<TransportMode>) -> TransportMode {
    match requested_mode {
        Some(TransportMode::Bicycle) => TransportMode::Bicycle,
        Some(TransportMode::Pedestrian) => TransportMode::Pedestrian,
        Some(TransportMode::Car) | None => TransportMode::Car,
        Some(unsupported_mode) => {
            log::warn!(
                "MapQuest routing does not route for {unsupported_mode:?}; using automobile"
            );
            TransportMode::Car
        }
    }
}

/// Selects the documented `routeType` query value.
pub(crate) fn route_type_query_text(
    transport_mode: TransportMode,
    route_strategy: Option<RouteStrategy>,
) -> &'static str {
    match transport_mode {
        TransportMode::Bicycle => BICYCLE_ROUTE_TYPE,
        TransportMode::Pedestrian => PEDESTRIAN_ROUTE_TYPE,
        _ => match route_strategy.unwrap_or_default() {
            RouteStrategy::Fastest => FASTEST_ROUTE_TYPE,
            RouteStrategy::Shortest => SHORTEST_ROUTE_TYPE,
        },
    }
}

/// Builds the documented `avoids` query value, when any feature applies.
fn avoid_query_text(avoid: &[AvoidType]) -> Option<String> {
    let mut avoid_features = Vec::new();
    for avoid_type in avoid {
        match avoid_type {
            AvoidType::Tolls => avoid_features.push(TOLL_ROAD_AVOID_FEATURE),
            AvoidType::Highways => avoid_features.push(LIMITED_ACCESS_AVOID_FEATURE),
            AvoidType::Ferries => avoid_features.push(FERRY_AVOID_FEATURE),
            AvoidType::DirtRoads => avoid_features.push(UNPAVED_AVOID_FEATURE),
            AvoidType::Tunnels => {
                log::warn!("MapQuest routing does not support avoiding tunnels; ignoring");
            }
        }
    }
    if avoid_features.is_empty() {
        None
    } else {
        Some(avoid_features.join(AVOID_FEATURE_SEPARATOR))
    }
}

/// Reports route options this request cannot express.
fn report_unsupported_options(options: &RouteOptions) {
    if options.alternatives.is_some() {
        log::warn!("MapQuest routing does not support alternative routes; ignoring");
    }
    if options.departure_time.is_some() || options.arrival_time.is_some() {
        log::warn!("MapQuest routing does not support departure or arrival times; ignoring");
    }
    if options.language.is_some() {
        log::warn!("MapQuest routing does not support a response language; ignoring");
    }
    if options.provider_extra.is_some() {
        log::warn!("MapQuest routing does not consume provider_extra; ignoring");
    }
}

/// Projects a decoded Directions API v2 route payload into the core response.
///
/// # Parameters
/// - `payload` — a decoded route response body carrying a `route` object.
/// - `transport_mode` — the mode the route was requested for; recorded on the
///   result.
///
/// # Returns
/// A response with the single projected route.
///
/// # Errors
/// Returns [`EveryMapError::ProviderError`] when the payload lacks the route,
/// its shape, its legs, a maneuver's `turnType` or its bounding box, or when a
/// coordinate cannot be constructed.
pub fn parse_route_payload(
    payload: &Value,
    transport_mode: TransportMode,
) -> EveryMapResult<RouteResponse> {
    let route = payload
        .get("route")
        .ok_or_else(|| invalid_response("route payload lacks the route object"))?;
    let shape_points = route
        .pointer("/shape/shapePoints")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("route payload lacks shape.shapePoints"))?;
    let route_shape = parse_shape_points(shape_points)?;
    let maneuver_indexes = read_index_list(route.pointer("/shape/maneuverIndexes"));
    let leg_indexes = read_index_list(route.pointer("/shape/legIndexes"));

    let legs = route
        .get("legs")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("route payload lacks legs"))?;
    let mut parsed_legs = Vec::with_capacity(legs.len());
    let mut flat_steps = Vec::new();
    let mut maneuver_offset_across_legs = 0_usize;
    for (leg_position, leg) in legs.iter().enumerate() {
        let maneuvers = leg
            .get("maneuvers")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_response("route leg lacks maneuvers"))?;
        let mut steps = Vec::with_capacity(maneuvers.len());
        for (maneuver_index_in_leg, maneuver) in maneuvers.iter().enumerate() {
            let global_maneuver_index = maneuver_offset_across_legs + maneuver_index_in_leg;
            steps.push(parse_maneuver_step(
                maneuver,
                maneuvers.get(maneuver_index_in_leg + 1),
                global_maneuver_index,
                &maneuver_indexes,
                &route_shape,
            )?);
        }
        maneuver_offset_across_legs += maneuvers.len();
        flat_steps.extend(steps.iter().cloned());
        parsed_legs.push(RouteLeg {
            distance: kilometers_to_meters(
                leg.get("distance")
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
            ),
            duration: leg.get("time").and_then(Value::as_f64).unwrap_or_default(),
            steps,
            shape: leg_shape(leg_position, &leg_indexes, &route_shape),
        });
    }

    let bounding_box = parse_route_bounding_box(route)?;
    Ok(RouteResponse {
        routes: vec![RouteResult {
            distance: kilometers_to_meters(
                route
                    .get("distance")
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
            ),
            duration: route
                .get("time")
                .and_then(Value::as_f64)
                .unwrap_or_default(),
            geometry: Polyline::new(route_shape),
            transport_mode: Some(transport_mode),
            steps: flat_steps,
            bounding_box: Some(bounding_box),
            raw: None,
            legs: parsed_legs,
        }],
    })
}

/// Parses one maneuver into the step whose shape runs from that maneuver's
/// first shape point to the next maneuver's start.
fn parse_maneuver_step(
    maneuver: &Value,
    following_maneuver: Option<&Value>,
    global_maneuver_index: usize,
    maneuver_indexes: &[usize],
    route_shape: &[Coordinate],
) -> EveryMapResult<RouteStep> {
    let narrative = maneuver
        .get("narrative")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let following_narrative = following_maneuver
        .and_then(|following_maneuver| following_maneuver.get("narrative"))
        .and_then(Value::as_str);
    let step_text = following_narrative
        .filter(|following_text| !following_text.is_empty())
        .unwrap_or(narrative)
        .to_string();
    let turn_type = maneuver
        .get("turnType")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid_response("maneuver lacks turnType"))?;
    let maneuver_type = if global_maneuver_index == 0 {
        ManeuverType::Departure
    } else if global_maneuver_index + 1 == maneuver_indexes.len() {
        ManeuverType::Arrival
    } else {
        map_turn_type_to_maneuver_type(turn_type, narrative)
    };
    let shape = maneuver_shape_slice(global_maneuver_index, maneuver_indexes, route_shape);
    let start_coordinate = maneuver.get("startPoint").and_then(parse_lat_lng);
    let end_coordinate = following_maneuver
        .and_then(|following_maneuver| following_maneuver.get("startPoint"))
        .and_then(parse_lat_lng)
        .or_else(|| shape.last().copied())
        .or_else(|| route_shape.last().copied());
    Ok(RouteStep {
        instruction: non_empty_owned(step_text.clone()),
        distance: maneuver
            .get("distance")
            .and_then(Value::as_f64)
            .map(kilometers_to_meters),
        duration: maneuver.get("time").and_then(Value::as_f64),
        start_coordinate,
        end_coordinate,
        maneuver_type: Some(maneuver_type),
        speech_instruction: non_empty_owned(step_text),
        lane_guidances: Vec::new(),
        shape,
    })
}

/// Returns the text unchanged when it is non-empty.
fn non_empty_owned(text: String) -> Option<String> {
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Splits the route-wide shape at maneuver boundaries.
fn maneuver_shape_slice(
    global_maneuver_index: usize,
    maneuver_indexes: &[usize],
    route_shape: &[Coordinate],
) -> Vec<Coordinate> {
    let Some(start_index) = maneuver_indexes.get(global_maneuver_index) else {
        return Vec::new();
    };
    let start_index = (*start_index).min(route_shape.len());
    let end_index = maneuver_indexes
        .get(global_maneuver_index + 1)
        .map_or(route_shape.len(), |next_index| {
            (*next_index).min(route_shape.len())
        });
    route_shape[start_index..end_index.max(start_index)].to_vec()
}

/// Splits the route-wide shape at leg boundaries when the payload reports
/// them.
fn leg_shape(
    leg_position: usize,
    leg_indexes: &[usize],
    route_shape: &[Coordinate],
) -> Vec<Coordinate> {
    let Some(start_index) = leg_indexes.get(leg_position) else {
        return Vec::new();
    };
    let start_index = (*start_index).min(route_shape.len());
    let end_index = leg_indexes
        .get(leg_position + 1)
        .map_or(route_shape.len(), |next_index| {
            (*next_index).min(route_shape.len())
        });
    route_shape[start_index..end_index.max(start_index)].to_vec()
}

/// Reads an index list, tolerating its absence.
fn read_index_list(value: Option<&Value>) -> Vec<usize> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_u64)
                .map(|index| usize::try_from(index).unwrap_or(usize::MAX))
                .collect()
        })
        .unwrap_or_default()
}

/// Parses the flat `lat, lng` shape array into coordinates.
fn parse_shape_points(shape_points: &[Value]) -> EveryMapResult<Vec<Coordinate>> {
    if shape_points.len() % 2 != 0 {
        return Err(invalid_response(
            "shapePoints has an odd element count and cannot form lat/lng pairs",
        ));
    }
    shape_points
        .chunks_exact(2)
        .map(|pair| {
            let latitude = pair[0]
                .as_f64()
                .ok_or_else(|| invalid_response("shape point latitude is not numeric"))?;
            let longitude = pair[1]
                .as_f64()
                .ok_or_else(|| invalid_response("shape point longitude is not numeric"))?;
            Coordinate::new(latitude, longitude)
                .map_err(|error| invalid_response(error.to_string()))
        })
        .collect()
}

/// Parses the route's `ul`/`lr` bounding box into the core's corner pair.
fn parse_route_bounding_box(route: &Value) -> EveryMapResult<BoundingBox> {
    let upper_left = route
        .pointer("/boundingBox/ul")
        .and_then(parse_lat_lng)
        .ok_or_else(|| invalid_response("route payload lacks boundingBox.ul"))?;
    let lower_right = route
        .pointer("/boundingBox/lr")
        .and_then(parse_lat_lng)
        .ok_or_else(|| invalid_response("route payload lacks boundingBox.lr"))?;
    let south_west = Coordinate::new(lower_right.lat, upper_left.lng)
        .map_err(|error| invalid_response(error.to_string()))?;
    let north_east = Coordinate::new(upper_left.lat, lower_right.lng)
        .map_err(|error| invalid_response(error.to_string()))?;
    Ok(BoundingBox::new(north_east, south_west))
}

/// Reads a `{lat, lng}` object.
fn parse_lat_lng(lat_lng: &Value) -> Option<Coordinate> {
    let latitude = lat_lng.get("lat")?.as_f64()?;
    let longitude = lat_lng.get("lng")?.as_f64()?;
    Coordinate::new(latitude, longitude).ok()
}

/// Converts a kilometre value to meters.
fn kilometers_to_meters(distance_kilometers: f64) -> f64 {
    distance_kilometers * METERS_PER_KILOMETER
}

/// Maps the documented Directions API `turnType` enumeration onto the core
/// maneuver type.
///
/// # Parameters
/// - `turn_type` — the numeric `turnType` value.
/// - `narrative` — the maneuver narrative; used to read a roundabout exit
///   ordinal.
///
/// # Returns
/// - [`ManeuverType::Arrival`] for `-1`.
/// - The corresponding [`ManeuverType::Turn`] for `0` through `9`.
/// - [`ManeuverType::Merge`], `OnRamp`, `OffRamp` or `Fork` for `10` through
///   `18`.
/// - [`ManeuverType::Roundabout`] for `24` through `29`, carrying the exit
///   ordinal when the narrative states one and the default exit index
///   otherwise.
/// - [`ManeuverType::Continue`] for any other value.
pub fn map_turn_type_to_maneuver_type(turn_type: i64, narrative: &str) -> ManeuverType {
    match turn_type {
        -1 => ManeuverType::Arrival,
        0 => ManeuverType::Turn(TurnDirection::Straight),
        1 => ManeuverType::Turn(TurnDirection::SlightRight),
        2 => ManeuverType::Turn(TurnDirection::Right),
        3 => ManeuverType::Turn(TurnDirection::SharpRight),
        4 | 8 | 9 => ManeuverType::Turn(TurnDirection::UTurn),
        5 => ManeuverType::Turn(TurnDirection::SharpLeft),
        6 => ManeuverType::Turn(TurnDirection::Left),
        7 => ManeuverType::Turn(TurnDirection::SlightLeft),
        10 | 11 => ManeuverType::Merge,
        12 | 13 => ManeuverType::OnRamp,
        14 | 15 => ManeuverType::OffRamp,
        16..=18 => ManeuverType::Fork,
        24..=29 => ManeuverType::Roundabout(
            extract_roundabout_exit_index(narrative).unwrap_or(DEFAULT_ROUNDABOUT_EXIT_INDEX),
        ),
        _ => ManeuverType::Continue,
    }
}

/// Reads the roundabout exit ordinal from a narrative such as
/// `"At the roundabout take the 3rd exit onto Broadway"`.
///
/// The service reports the traversal but not the exit index as data, so the
/// ordinal is read from the text preceding the word `exit`.
///
/// # Parameters
/// - `narrative` — the maneuver narrative.
///
/// # Returns
/// The parsed exit index, or `None` when the narrative states none.
pub fn extract_roundabout_exit_index(narrative: &str) -> Option<u8> {
    let lowercase = narrative.to_lowercase();
    let exit_position = lowercase.find("exit")?;
    let prefix = &lowercase[..exit_position];
    let digits_end = prefix.rfind(|character: char| character.is_ascii_digit())?;
    let digits_start = prefix[..=digits_end]
        .rfind(|character: char| !character.is_ascii_digit())
        .map_or(0, |position| position + 1);
    prefix[digits_start..=digits_end].parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use super::{
        extract_roundabout_exit_index, map_turn_type_to_maneuver_type, parse_route_payload,
        route_type_query_text,
    };
    use everymap_core::domains::routing::{
        ManeuverType, RouteStrategy, TransportMode, TurnDirection,
    };
    use serde_json::{json, Value};

    fn fixture(name: &str) -> Value {
        let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("the fixture must be readable"),
        )
        .expect("the fixture must be valid JSON")
    }

    #[test]
    fn a_recorded_two_waypoint_route_projects_legs_steps_and_geometry() {
        let response = parse_route_payload(
            &fixture("directions_route_two_waypoints.json"),
            TransportMode::Car,
        )
        .expect("the recorded route must project");
        assert_eq!(response.routes.len(), 1);
        let route = &response.routes[0];
        assert_eq!(route.legs.len(), 1);
        assert_eq!(route.steps.len(), 6);
        let leg = &route.legs[0];
        assert_eq!(leg.steps.len(), 6);
        assert!(!leg.shape.is_empty(), "the leg carries its shape");
        assert_eq!(route.geometry.points.len(), 45);
        assert!((route.distance - 1700.0).abs() < 50.0);
        assert_eq!(route.duration, 235.0);
        assert_eq!(route.transport_mode, Some(TransportMode::Car));
        assert!(route.bounding_box.is_some());
        assert_eq!(leg.steps[0].maneuver_type, Some(ManeuverType::Departure));
        assert_eq!(leg.steps[5].maneuver_type, Some(ManeuverType::Arrival));
        assert_eq!(
            leg.steps[1].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Left))
        );
        assert_eq!(
            leg.steps[3].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Right))
        );
        assert_eq!(
            leg.steps[4].maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Right))
        );
        for step in &leg.steps {
            assert!(!step.shape.is_empty(), "every recorded step has a shape");
            assert!(step.lane_guidances.is_empty());
        }
    }

    #[test]
    fn step_texts_carry_the_maneuver_at_the_step_end() {
        let response = parse_route_payload(
            &fixture("directions_route_two_waypoints.json"),
            TransportMode::Car,
        )
        .expect("the recorded route must project");
        let steps = &response.routes[0].legs[0].steps;
        assert_eq!(
            steps[0].instruction.as_deref(),
            Some("Turn left onto L St. Go for 864 m.")
        );
        assert_eq!(
            steps[0].speech_instruction.as_deref(),
            Some("Turn left onto L St. Go for 864 m.")
        );
        assert_eq!(steps[5].instruction.as_deref(), Some("Arrive at Front St."));
    }

    #[test]
    fn a_recorded_route_reports_start_and_end_coordinates() {
        let response = parse_route_payload(
            &fixture("directions_route_two_waypoints.json"),
            TransportMode::Car,
        )
        .expect("the recorded route must project");
        let first_step = &response.routes[0].legs[0].steps[0];
        assert!(first_step.start_coordinate.is_some());
        assert!(first_step.end_coordinate.is_some());
        let last_step = &response.routes[0].legs[0].steps[5];
        assert!(last_step.start_coordinate.is_some());
        assert!(last_step.end_coordinate.is_some());
    }

    #[test]
    fn the_documented_turn_type_table_maps_completely() {
        assert_eq!(
            map_turn_type_to_maneuver_type(0, ""),
            ManeuverType::Turn(TurnDirection::Straight)
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(1, ""),
            ManeuverType::Turn(TurnDirection::SlightRight)
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(3, ""),
            ManeuverType::Turn(TurnDirection::SharpRight)
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(4, ""),
            ManeuverType::Turn(TurnDirection::UTurn)
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(6, ""),
            ManeuverType::Turn(TurnDirection::Left)
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(7, ""),
            ManeuverType::Turn(TurnDirection::SlightLeft)
        );
        assert_eq!(map_turn_type_to_maneuver_type(10, ""), ManeuverType::Merge);
        assert_eq!(map_turn_type_to_maneuver_type(12, ""), ManeuverType::OnRamp);
        assert_eq!(
            map_turn_type_to_maneuver_type(14, ""),
            ManeuverType::OffRamp
        );
        assert_eq!(map_turn_type_to_maneuver_type(16, ""), ManeuverType::Fork);
        assert_eq!(
            map_turn_type_to_maneuver_type(-1, ""),
            ManeuverType::Arrival
        );
        assert_eq!(
            map_turn_type_to_maneuver_type(19, ""),
            ManeuverType::Continue
        );
    }

    #[test]
    fn a_roundabout_exit_index_is_read_from_the_narrative() {
        assert_eq!(
            map_turn_type_to_maneuver_type(24, "At the roundabout take the 3rd exit onto Broadway"),
            ManeuverType::Roundabout(3)
        );
        assert_eq!(extract_roundabout_exit_index("take the 2nd exit"), Some(2));
        assert_eq!(extract_roundabout_exit_index("no ordinal here"), None);
        assert_eq!(
            map_turn_type_to_maneuver_type(25, "At the roundabout take an exit"),
            ManeuverType::Roundabout(1)
        );
    }

    #[test]
    fn the_strategy_selects_the_automobile_route_type() {
        assert_eq!(
            route_type_query_text(TransportMode::Car, Some(RouteStrategy::Fastest)),
            "fastest"
        );
        assert_eq!(
            route_type_query_text(TransportMode::Car, Some(RouteStrategy::Shortest)),
            "shortest"
        );
        assert_eq!(route_type_query_text(TransportMode::Car, None), "fastest");
        assert_eq!(
            route_type_query_text(TransportMode::Bicycle, Some(RouteStrategy::Shortest)),
            "bicycle"
        );
        assert_eq!(
            route_type_query_text(TransportMode::Pedestrian, None),
            "pedestrian"
        );
    }

    #[test]
    fn an_odd_shape_point_count_is_rejected() {
        let payload = json!({
            "route": {
                "distance": 1.0,
                "time": 1,
                "shape": {"shapePoints": [1.0]},
                "legs": [],
                "boundingBox": {
                    "ul": {"lat": 1.0, "lng": 1.0},
                    "lr": {"lat": 0.0, "lng": 0.0}
                }
            }
        });
        assert!(parse_route_payload(&payload, TransportMode::Car).is_err());
    }

    #[test]
    fn a_missing_route_object_is_rejected() {
        assert!(parse_route_payload(&json!({}), TransportMode::Car).is_err());
    }
}
