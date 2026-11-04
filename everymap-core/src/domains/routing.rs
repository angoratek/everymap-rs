use crate::error::{EveryMapError, EveryMapResult};
use crate::types::{BoundingBox, Coordinate, Polyline};
use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl fmt::Display for DepartureTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepartureTime::Now => f.write_str("now"),
            DepartureTime::Timestamp(ts) => write!(f, "{}", ts),
            DepartureTime::Iso8601(s) => f.write_str(s),
        }
    }
}

/// Typed departure time for routing, matching, and isoline requests.
///
/// Replaces fragile `Option<String>` guessing with an explicit enum.
/// Serializes as a plain string for backward compatibility.
#[derive(Debug, Clone, PartialEq)]
pub enum DepartureTime {
    /// Depart now.
    Now,
    /// Unix timestamp in seconds.
    Timestamp(i64),
    /// ISO 8601 formatted date/time string.
    Iso8601(String),
}

impl Serialize for DepartureTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            DepartureTime::Now => "now".to_string(),
            DepartureTime::Timestamp(ts) => ts.to_string(),
            DepartureTime::Iso8601(s) => s.clone(),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for DepartureTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s == "now" {
            Ok(DepartureTime::Now)
        } else if let Ok(ts) = s.parse::<i64>() {
            Ok(DepartureTime::Timestamp(ts))
        } else {
            Ok(DepartureTime::Iso8601(s))
        }
    }
}

/// Avoid types for routing restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AvoidType {
    Tolls,
    Ferries,
    Tunnels,
    Highways,
    DirtRoads,
}

/// Unit system a route request asks the provider to express distances in.
///
/// The core records distances in meters regardless of this choice; a provider
/// asks its service for the requested unit and converts the answer back. A
/// provider that only serves one unit reports so by ignoring the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    /// Kilometers or meters, the default.
    #[default]
    Metric,
    /// Miles or feet.
    Imperial,
}

/// Options for route calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteOptions {
    /// Transport mode for the route
    pub transport_mode: Option<TransportMode>,
    /// Number of alternative routes to compute
    pub alternatives: Option<u32>,
    /// Route restrictions (tolls, ferries, highways, etc.)
    pub avoid: Vec<AvoidType>,
    /// Departure time
    pub departure_time: Option<DepartureTime>,
    /// Arrival time
    pub arrival_time: Option<DepartureTime>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// What the route is optimized for. `None` leaves the choice to the
    /// provider; a provider that cannot honour [`RouteStrategy::Shortest`]
    /// reports so through [`Router::route_capabilities`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_strategy: Option<RouteStrategy>,
    /// The unit system the provider should express distances in. The core
    /// result stays in meters; a provider that serves one unit ignores this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_system: Option<UnitSystem>,
    /// Provider-specific options (HERE: routing_mode, spans, truck params; Google: waypoints, traffic_model)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Transport mode for a route.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransportMode {
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Scooter,
    Bus,
    Taxi,
    Unknown,
}

/// What a routing request optimizes for.
///
/// A provider ranks candidate paths by a cost function; the strategy selects
/// between travel time and physical length. Not every provider exposes the
/// choice, which is why [`RouteCapabilities`] reports whether the shortest
/// strategy can be honoured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteStrategy {
    /// Minimize expected travel time.
    #[default]
    Fastest,
    /// Minimize traveled distance.
    Shortest,
}

/// What a routing provider can honour of the transport-mode and strategy
/// choices a request may carry.
///
/// A surface reads this answer to disable a choice with a reason instead of
/// sending the request and receiving a route from a path the reader did not
/// ask for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteCapabilities {
    /// The transport modes the provider can route for.
    pub supported_transport_modes: Vec<TransportMode>,
    /// Whether the provider can minimize traveled distance instead of travel
    /// time.
    pub supports_shortest_route_strategy: bool,
}

impl RouteCapabilities {
    /// Every mode and both strategies: the answer of a provider that honours
    /// every choice the contract can express.
    pub fn all_supported() -> Self {
        Self {
            supported_transport_modes: vec![
                TransportMode::Car,
                TransportMode::Truck,
                TransportMode::Pedestrian,
                TransportMode::Bicycle,
                TransportMode::Scooter,
                TransportMode::Bus,
                TransportMode::Taxi,
            ],
            supports_shortest_route_strategy: true,
        }
    }

    /// Whether the provider can route for one transport mode.
    pub fn supports_transport_mode(&self, transport_mode: TransportMode) -> bool {
        self.supported_transport_modes.contains(&transport_mode)
    }
}

/// Turn direction reported by a maneuver or permitted by a lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnDirection {
    /// Continue straight ahead.
    Straight,
    /// Sharp left turn.
    SharpLeft,
    /// Standard left turn.
    Left,
    /// Gentle left turn.
    SlightLeft,
    /// Gentle right turn.
    SlightRight,
    /// Standard right turn.
    Right,
    /// Sharp right turn.
    SharpRight,
    /// Full reversal of direction.
    UTurn,
    /// Leave the current road.
    LeavingRoad,
}

/// High-level maneuver classification carried by a [`RouteStep`].
///
/// The classification serializes as its [`Self::stable_label`], the single
/// spelling every provider in the workspace uses. A route serialized by one
/// component and deserialized by another therefore round-trips without
/// spelling drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManeuverType {
    /// Route start.
    Departure,
    /// Route end.
    Arrival,
    /// Road-name or road-class change without a turn.
    Continue,
    /// Left, right, slight or sharp turn; see [`TurnDirection`].
    Turn(TurnDirection),
    /// Merge onto another road.
    Merge,
    /// Motorway on-ramp.
    OnRamp,
    /// Motorway off-ramp.
    OffRamp,
    /// Fork in the road.
    Fork,
    /// Roundabout traversal; carries the exit index starting at one.
    Roundabout(u8),
    /// Transition to a ferry.
    Ferry,
}

/// The label prefix of turn maneuvers in the stable spelling.
const TURN_STABLE_LABEL_PREFIX: &str = "turn";
/// The label prefix of roundabout maneuvers in the stable spelling.
const ROUNDABOUT_STABLE_LABEL_PREFIX: &str = "roundabout";
/// The character joining a maneuver label to its embedded parameter.
const STABLE_LABEL_PARAMETER_SEPARATOR: char = '-';
/// The lowest one-based exit index a roundabout label may carry.
const STABLE_LABEL_ROUNDABOUT_MINIMUM_EXIT_INDEX: u8 = 1;
/// The serde expectation naming the stable spelling for a decoder.
const STABLE_LABEL_SERDE_EXPECTATION: &str =
    "a stable maneuver label such as \"departure\", \"turn-left\" or \"roundabout-2\"";

/// The kebab-case parameter segment a turn direction contributes to its
/// maneuver label (`left`, `sharp-left`, `u-turn`, ...).
fn turn_direction_label_segment(turn_direction: TurnDirection) -> &'static str {
    match turn_direction {
        TurnDirection::Straight => "straight",
        TurnDirection::SharpLeft => "sharp-left",
        TurnDirection::Left => "left",
        TurnDirection::SlightLeft => "slight-left",
        TurnDirection::SlightRight => "slight-right",
        TurnDirection::Right => "right",
        TurnDirection::SharpRight => "sharp-right",
        TurnDirection::UTurn => "u-turn",
        TurnDirection::LeavingRoad => "leaving-road",
    }
}

/// Builds the turn direction a kebab-case parameter segment names; the
/// inverse of [`turn_direction_label_segment`].
fn turn_direction_from_label_segment(label_segment: &str) -> Option<TurnDirection> {
    Some(match label_segment {
        "straight" => TurnDirection::Straight,
        "sharp-left" => TurnDirection::SharpLeft,
        "left" => TurnDirection::Left,
        "slight-left" => TurnDirection::SlightLeft,
        "slight-right" => TurnDirection::SlightRight,
        "right" => TurnDirection::Right,
        "sharp-right" => TurnDirection::SharpRight,
        "u-turn" => TurnDirection::UTurn,
        "leaving-road" => TurnDirection::LeavingRoad,
        _ => return None,
    })
}

/// Builds the parameterized maneuvers (`Turn(...)`, `Roundabout(...)`) a
/// stable label names; `None` for anything that is not a well-formed
/// parameterized label of those kinds.
fn parameterized_from_stable_label(stable_label: &str) -> Option<ManeuverType> {
    let label_parameter = |label_prefix: &str| {
        stable_label
            .strip_prefix(label_prefix)
            .and_then(|rest| rest.strip_prefix(STABLE_LABEL_PARAMETER_SEPARATOR))
    };
    if let Some(turn_direction) =
        label_parameter(TURN_STABLE_LABEL_PREFIX).and_then(turn_direction_from_label_segment)
    {
        return Some(ManeuverType::Turn(turn_direction));
    }
    label_parameter(ROUNDABOUT_STABLE_LABEL_PREFIX)
        .and_then(|exit_index_text| exit_index_text.parse::<u8>().ok())
        .filter(|exit_index| *exit_index >= STABLE_LABEL_ROUNDABOUT_MINIMUM_EXIT_INDEX)
        .map(ManeuverType::Roundabout)
}

impl ManeuverType {
    /// The stable kebab-case label of this maneuver: `departure`,
    /// `turn-left`, `roundabout-2`, and so on.
    pub fn stable_label(&self) -> String {
        match self {
            Self::Departure => "departure".to_string(),
            Self::Arrival => "arrival".to_string(),
            Self::Continue => "continue".to_string(),
            Self::Turn(turn_direction) => format!(
                "{TURN_STABLE_LABEL_PREFIX}{STABLE_LABEL_PARAMETER_SEPARATOR}{}",
                turn_direction_label_segment(*turn_direction)
            ),
            Self::Merge => "merge".to_string(),
            Self::OnRamp => "on-ramp".to_string(),
            Self::OffRamp => "off-ramp".to_string(),
            Self::Fork => "fork".to_string(),
            Self::Roundabout(exit_index) => format!(
                "{ROUNDABOUT_STABLE_LABEL_PREFIX}{STABLE_LABEL_PARAMETER_SEPARATOR}{exit_index}"
            ),
            Self::Ferry => "ferry".to_string(),
        }
    }

    /// Builds the maneuver a stable label (see [`Self::stable_label`]) names.
    ///
    /// Returns `None` for an unknown or malformed label; no label is guessed.
    pub fn from_stable_label(stable_label: &str) -> Option<Self> {
        match stable_label {
            "departure" => Some(Self::Departure),
            "arrival" => Some(Self::Arrival),
            "continue" => Some(Self::Continue),
            "merge" => Some(Self::Merge),
            "on-ramp" => Some(Self::OnRamp),
            "off-ramp" => Some(Self::OffRamp),
            "fork" => Some(Self::Fork),
            "ferry" => Some(Self::Ferry),
            _ => parameterized_from_stable_label(stable_label),
        }
    }
}

/// Maneuvers serialize as their stable label.
impl Serialize for ManeuverType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.stable_label())
    }
}

/// Maneuvers deserialize from their stable label. Any other form is a
/// decoding error, not a guess.
impl<'de> Deserialize<'de> for ManeuverType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StableLabelVisitor;

        impl serde::de::Visitor<'_> for StableLabelVisitor {
            type Value = ManeuverType;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(STABLE_LABEL_SERDE_EXPECTATION)
            }

            fn visit_str<E: serde::de::Error>(self, label: &str) -> Result<Self::Value, E> {
                ManeuverType::from_stable_label(label).ok_or_else(|| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(label),
                        &STABLE_LABEL_SERDE_EXPECTATION,
                    )
                })
            }
        }

        deserializer.deserialize_str(StableLabelVisitor)
    }
}

/// Recommended lane(s) for an upcoming maneuver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneGuidance {
    /// Zero-based lane index across the road cross-section.
    pub lane_index: u8,
    /// Directions this lane can legally lead to.
    pub permitted_directions: Vec<TurnDirection>,
    /// Whether this lane is among the recommended ones.
    pub is_recommended: bool,
}

/// A single leg/step in a route (e.g., a turn-by-turn instruction).
///
/// The first five fields are the original flat projection and stay populated
/// by every provider. The navigation-grade fields are optional so a provider
/// that cannot report them still produces a valid step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    /// Instruction text (e.g., "Turn right onto Main St")
    pub instruction: Option<String>,
    /// Distance of this step in meters
    pub distance: Option<f64>,
    /// Duration of this step in seconds
    pub duration: Option<f64>,
    /// Starting coordinate of this step
    pub start_coordinate: Option<Coordinate>,
    /// Ending coordinate of this step
    pub end_coordinate: Option<Coordinate>,
    /// Maneuver classification, when the provider reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maneuver_type: Option<ManeuverType>,
    /// Instruction tuned for text-to-speech engines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_instruction: Option<String>,
    /// Lane guidance available at the maneuver, oldest lane first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lane_guidances: Vec<LaneGuidance>,
    /// Step geometry from the previous step end to this maneuver point.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape: Vec<Coordinate>,
}

/// Leg between two consecutive waypoints.
///
/// A two-endpoint route has exactly one leg; a route through ordered
/// waypoints has one leg per consecutive pair, in request order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLeg {
    /// Leg distance in meters.
    pub distance: f64,
    /// Leg travel time in seconds.
    pub duration: f64,
    /// Maneuver steps in traversal order.
    pub steps: Vec<RouteStep>,
    /// Leg geometry as an ordered polyline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape: Vec<Coordinate>,
}

/// A unified route result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    /// Total route distance in meters
    pub distance: f64,
    /// Total route duration in seconds
    pub duration: f64,
    /// Route geometry as a polyline
    pub geometry: Polyline,
    /// Transport mode used for this route
    pub transport_mode: Option<TransportMode>,
    /// Flat turn-by-turn steps. Kept for the two-endpoint projection; a
    /// navigation-grade provider also fills [`Self::legs`].
    pub steps: Vec<RouteStep>,
    /// Bounding box for the route (if available)
    pub bounding_box: Option<BoundingBox>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
    /// Legs, one per consecutive waypoint pair. Empty when the provider only
    /// reports the flat [`Self::steps`] projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legs: Vec<RouteLeg>,
}

/// A unified route response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    /// The route results (may contain alternatives)
    pub routes: Vec<RouteResult>,
}

/// Domain name reported when a router cannot honour intermediate waypoints.
pub const INTERMEDIATE_WAYPOINTS_DOMAIN: &str = "routing with intermediate waypoints";

#[async_trait]
pub trait Router: Send + Sync {
    /// Calculates a route between two endpoints.
    ///
    /// This is the original two-endpoint entry point. It stays part of the
    /// contract so existing implementations keep compiling; new callers that
    /// need ordered waypoints use [`Self::calculate_route_with_waypoints`].
    async fn calculate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse>;

    /// Calculates a route through ordered waypoints.
    ///
    /// # Parameters
    /// - `waypoints` — the ordered stops, starting at the origin and ending
    ///   at the destination. At least two are required.
    /// - `options` — transport mode, strategy, avoidance and timing choices.
    ///
    /// # Returns
    /// A response with one route per alternative. Each route carries one leg
    /// per consecutive waypoint pair in request order.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when fewer than two
    /// waypoints are supplied. Implementations that only route between two
    /// endpoints keep the default body, which delegates a two-waypoint
    /// request to [`Self::calculate_route`] and reports
    /// [`EveryMapError::UnsupportedDomain`] with
    /// [`INTERMEDIATE_WAYPOINTS_DOMAIN`] for anything longer.
    async fn calculate_route_with_waypoints(
        &self,
        waypoints: &[Coordinate],
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        if waypoints.len() < 2 {
            return Err(EveryMapError::ValidationError(
                "routing requires at least an origin and a destination".to_string(),
            ));
        }
        let [start, end] = waypoints else {
            return Err(EveryMapError::unsupported_domain(
                "the router",
                INTERMEDIATE_WAYPOINTS_DOMAIN,
            ));
        };
        self.calculate_route(start, end, options).await
    }

    /// Reports which transport modes and route strategies this provider can
    /// honour.
    ///
    /// The default reports every mode and both strategies, so an existing
    /// implementation keeps compiling unchanged; a provider that cannot serve
    /// a choice overrides this.
    fn route_capabilities(&self) -> RouteCapabilities {
        RouteCapabilities::all_supported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_system_defaults_to_metric_and_serializes() {
        assert_eq!(UnitSystem::default(), UnitSystem::Metric);
        assert_eq!(
            serde_json::to_string(&UnitSystem::Imperial).unwrap(),
            "\"imperial\""
        );
    }

    #[test]
    fn test_route_options_default() {
        let options = RouteOptions::default();
        assert!(options.transport_mode.is_none());
        assert!(options.alternatives.is_none());
        assert!(options.avoid.is_empty());
        assert!(options.departure_time.is_none());
        assert!(options.arrival_time.is_none());
        assert!(options.language.is_none());
        assert!(options.provider_extra.is_none());
        assert!(options.route_strategy.is_none());
    }

    #[test]
    fn test_route_options_with_fields() {
        let options = RouteOptions {
            unit_system: None,
            transport_mode: Some(TransportMode::Car),
            alternatives: Some(3),
            avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
            language: Some("de-DE".to_string()),
            provider_extra: Some(serde_json::json!({"routing_mode": "fast"})),
            ..Default::default()
        };
        assert_eq!(options.transport_mode, Some(TransportMode::Car));
        assert_eq!(options.alternatives, Some(3));
        assert_eq!(options.avoid.len(), 2);
    }

    #[test]
    fn test_transport_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&TransportMode::Car).unwrap(),
            "\"Car\""
        );
        let mode: TransportMode = serde_json::from_str("\"Truck\"").unwrap();
        assert_eq!(mode, TransportMode::Truck);
    }

    #[test]
    fn test_avoid_type_serialization() {
        assert_eq!(
            serde_json::to_string(&AvoidType::Tolls).unwrap(),
            "\"Tolls\""
        );
        let avoid: AvoidType = serde_json::from_str("\"Highways\"").unwrap();
        assert_eq!(avoid, AvoidType::Highways);
    }

    #[test]
    fn test_route_result_construction() {
        let result = RouteResult {
            distance: 15000.0,
            duration: 1800.0,
            geometry: Polyline::new(vec![]),
            transport_mode: Some(TransportMode::Car),
            steps: vec![],
            bounding_box: None,
            raw: None,
            legs: vec![],
        };
        assert_eq!(result.distance, 15000.0);
        assert_eq!(result.duration, 1800.0);
        assert_eq!(result.transport_mode, Some(TransportMode::Car));
    }

    // --- AvoidType all 5 variants serde roundtrip ---

    #[test]
    fn test_avoid_type_all_variants_serde() {
        let variants = [
            AvoidType::Tolls,
            AvoidType::Ferries,
            AvoidType::Tunnels,
            AvoidType::Highways,
            AvoidType::DirtRoads,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: AvoidType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_avoid_type_all_variants_distinct() {
        let variants = [
            AvoidType::Tolls,
            AvoidType::Ferries,
            AvoidType::Tunnels,
            AvoidType::Highways,
            AvoidType::DirtRoads,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- TransportMode all 8 variants serde roundtrip ---

    #[test]
    fn test_transport_mode_all_variants_serde() {
        let variants = [
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
            TransportMode::Unknown,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: TransportMode = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_transport_mode_all_variants_distinct() {
        let variants = [
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
            TransportMode::Unknown,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- RouteResponse serde roundtrip ---

    #[test]
    fn test_route_response_serde_roundtrip() {
        let response = RouteResponse {
            routes: vec![RouteResult {
                distance: 15000.0,
                duration: 1800.0,
                geometry: Polyline::new(vec![]),
                transport_mode: Some(TransportMode::Car),
                steps: vec![],
                bounding_box: None,
                raw: None,
                legs: vec![],
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: RouteResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.routes.len(), 1);
        assert_eq!(back.routes[0].distance, 15000.0);
        assert_eq!(back.routes[0].transport_mode, Some(TransportMode::Car));
    }

    #[test]
    fn test_route_response_empty_routes() {
        let response = RouteResponse { routes: vec![] };
        let json = serde_json::to_string(&response).unwrap();
        let back: RouteResponse = serde_json::from_str(&json).unwrap();
        assert!(back.routes.is_empty());
    }

    // --- RouteOptions with provider_extra serde roundtrip ---

    #[test]
    fn test_route_options_serde_roundtrip() {
        let options = RouteOptions {
            transport_mode: Some(TransportMode::Truck),
            alternatives: Some(2),
            avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
            departure_time: Some(DepartureTime::Iso8601("2024-06-01T08:00:00".to_string())),
            arrival_time: None,
            language: Some("en".to_string()),
            route_strategy: Some(RouteStrategy::Shortest),
            unit_system: None,
            provider_extra: Some(
                serde_json::json!({"routing_mode": "fast", "truck": {"weight": 18}}),
            ),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: RouteOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transport_mode, Some(TransportMode::Truck));
        assert_eq!(back.alternatives, Some(2));
        assert_eq!(back.avoid.len(), 2);
        assert!(back.provider_extra.is_some());
        assert_eq!(back.route_strategy, Some(RouteStrategy::Shortest));
    }

    // --- RouteStep serde roundtrip ---

    #[test]
    fn test_route_step_serde_roundtrip() {
        let step = RouteStep {
            instruction: Some("Turn right onto Main St".to_string()),
            distance: Some(500.0),
            duration: Some(60.0),
            start_coordinate: Some(Coordinate::new(52.5, 13.4).unwrap()),
            end_coordinate: Some(Coordinate::new(52.51, 13.41).unwrap()),
            maneuver_type: Some(ManeuverType::Turn(TurnDirection::Right)),
            speech_instruction: Some("Turn right onto Main Street.".to_string()),
            lane_guidances: vec![],
            shape: vec![],
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: RouteStep = serde_json::from_str(&json).unwrap();
        assert_eq!(back.instruction.as_deref(), Some("Turn right onto Main St"));
        assert_eq!(back.distance, Some(500.0));
        assert_eq!(back.duration, Some(60.0));
        assert_eq!(
            back.maneuver_type,
            Some(ManeuverType::Turn(TurnDirection::Right))
        );
        assert_eq!(
            back.speech_instruction.as_deref(),
            Some("Turn right onto Main Street.")
        );
    }

    // --- RouteResult with all fields populated ---

    #[test]
    fn test_route_result_full_serde_roundtrip() {
        let result = RouteResult {
            distance: 0.0,
            duration: 0.0,
            geometry: Polyline::new(vec![Coordinate::new(52.5, 13.4).unwrap()]),
            transport_mode: Some(TransportMode::Pedestrian),
            steps: vec![RouteStep {
                instruction: Some("Walk north".to_string()),
                distance: Some(100.0),
                duration: Some(120.0),
                start_coordinate: None,
                end_coordinate: None,
                maneuver_type: None,
                speech_instruction: None,
                lane_guidances: vec![],
                shape: vec![],
            }],
            bounding_box: Some(BoundingBox::new(
                Coordinate::new(52.6, 13.5).unwrap(),
                Coordinate::new(52.4, 13.3).unwrap(),
            )),
            raw: Some(serde_json::json!({"legs": []})),
            legs: vec![RouteLeg {
                distance: 100.0,
                duration: 120.0,
                steps: vec![],
                shape: vec![],
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: RouteResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.distance, 0.0);
        assert_eq!(back.steps.len(), 1);
        assert!(back.bounding_box.is_some());
        assert!(back.raw.is_some());
        assert_eq!(back.legs.len(), 1);
    }

    // --- Edge cases ---

    #[test]
    fn test_route_result_zero_distance_duration() {
        let result = RouteResult {
            distance: 0.0,
            duration: 0.0,
            geometry: Polyline::new(vec![]),
            transport_mode: None,
            steps: vec![],
            bounding_box: None,
            raw: None,
            legs: vec![],
        };
        assert_eq!(result.distance, 0.0);
        assert_eq!(result.duration, 0.0);
    }

    #[test]
    fn test_route_options_all_avoid_types() {
        let options = RouteOptions {
            avoid: vec![
                AvoidType::Tolls,
                AvoidType::Ferries,
                AvoidType::Tunnels,
                AvoidType::Highways,
                AvoidType::DirtRoads,
            ],
            ..Default::default()
        };
        assert_eq!(options.avoid.len(), 5);
    }

    // --- DepartureTime serde ---

    #[test]
    fn test_departure_time_now_serde() {
        let json = serde_json::to_string(&DepartureTime::Now).unwrap();
        assert_eq!(json, "\"now\"");
        let back: DepartureTime = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DepartureTime::Now);
    }

    #[test]
    fn test_departure_time_timestamp_serde() {
        let dt = DepartureTime::Timestamp(1717200000);
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"1717200000\"");
        let back: DepartureTime = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DepartureTime::Timestamp(1717200000));
    }

    #[test]
    fn test_departure_time_iso8601_serde() {
        let dt = DepartureTime::Iso8601("2024-06-01T08:00:00Z".to_string());
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"2024-06-01T08:00:00Z\"");
        let back: DepartureTime = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back,
            DepartureTime::Iso8601("2024-06-01T08:00:00Z".to_string())
        );
    }

    // --- Navigation-grade contract ---

    #[test]
    fn test_route_strategy_serde_and_default() {
        assert_eq!(
            serde_json::to_string(&RouteStrategy::Shortest).unwrap(),
            "\"shortest\""
        );
        let back: RouteStrategy = serde_json::from_str("\"fastest\"").unwrap();
        assert_eq!(back, RouteStrategy::Fastest);
        assert_eq!(RouteStrategy::default(), RouteStrategy::Fastest);
    }

    #[test]
    fn test_route_capabilities_all_supported() {
        let capabilities = RouteCapabilities::all_supported();
        for mode in [
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
        ] {
            assert!(capabilities.supports_transport_mode(mode));
        }
        assert!(!capabilities.supports_transport_mode(TransportMode::Unknown));
        assert!(capabilities.supports_shortest_route_strategy);
    }

    #[test]
    fn test_maneuver_stable_labels_roundtrip() {
        let maneuvers = [
            ManeuverType::Departure,
            ManeuverType::Arrival,
            ManeuverType::Continue,
            ManeuverType::Turn(TurnDirection::Straight),
            ManeuverType::Turn(TurnDirection::SharpLeft),
            ManeuverType::Turn(TurnDirection::Left),
            ManeuverType::Turn(TurnDirection::SlightLeft),
            ManeuverType::Turn(TurnDirection::SlightRight),
            ManeuverType::Turn(TurnDirection::Right),
            ManeuverType::Turn(TurnDirection::SharpRight),
            ManeuverType::Turn(TurnDirection::UTurn),
            ManeuverType::Turn(TurnDirection::LeavingRoad),
            ManeuverType::Merge,
            ManeuverType::OnRamp,
            ManeuverType::OffRamp,
            ManeuverType::Fork,
            ManeuverType::Roundabout(2),
            ManeuverType::Ferry,
        ];
        for maneuver in maneuvers {
            let label = maneuver.stable_label();
            assert_eq!(ManeuverType::from_stable_label(&label), Some(maneuver));
            let json = serde_json::to_string(&maneuver).unwrap();
            assert_eq!(json, format!("\"{label}\""));
            let back: ManeuverType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, maneuver);
        }
    }

    #[test]
    fn test_maneuver_unknown_labels_are_not_guessed() {
        for label in ["left", "turn-", "roundabout-0", "roundabout-x", "u-turn"] {
            assert_eq!(ManeuverType::from_stable_label(label), None);
            assert!(serde_json::from_str::<ManeuverType>(&format!("\"{label}\"")).is_err());
        }
    }

    #[test]
    fn test_lane_guidance_serde_roundtrip() {
        let guidance = LaneGuidance {
            lane_index: 1,
            permitted_directions: vec![TurnDirection::Left, TurnDirection::UTurn],
            is_recommended: true,
        };
        let json = serde_json::to_string(&guidance).unwrap();
        assert!(json.contains("\"lane_index\":1"));
        assert!(json.contains("\"left\""));
        assert!(json.contains("\"u_turn\""));
        let back: LaneGuidance = serde_json::from_str(&json).unwrap();
        assert_eq!(back, guidance);
    }

    #[test]
    fn test_route_step_legacy_json_defaults_navigation_fields() {
        let legacy = serde_json::json!({
            "instruction": "Continue",
            "distance": 10.0,
            "duration": 2.0,
            "start_coordinate": null,
            "end_coordinate": null
        });
        let step: RouteStep = serde_json::from_value(legacy).unwrap();
        assert!(step.maneuver_type.is_none());
        assert!(step.speech_instruction.is_none());
        assert!(step.lane_guidances.is_empty());
        assert!(step.shape.is_empty());
    }

    #[test]
    fn test_route_result_legacy_json_defaults_legs() {
        let legacy = serde_json::json!({
            "distance": 1.0,
            "duration": 2.0,
            "geometry": {"points": []},
            "transport_mode": null,
            "steps": [],
            "bounding_box": null
        });
        let result: RouteResult = serde_json::from_value(legacy).unwrap();
        assert!(result.legs.is_empty());
    }

    #[test]
    fn test_route_leg_serde_roundtrip() {
        let leg = RouteLeg {
            distance: 1200.0,
            duration: 300.0,
            steps: vec![RouteStep {
                instruction: Some("Take the roundabout".to_string()),
                distance: Some(40.0),
                duration: Some(12.0),
                start_coordinate: None,
                end_coordinate: None,
                maneuver_type: Some(ManeuverType::Roundabout(2)),
                speech_instruction: Some("Take the second exit at the roundabout.".to_string()),
                lane_guidances: vec![LaneGuidance {
                    lane_index: 0,
                    permitted_directions: vec![TurnDirection::Straight],
                    is_recommended: true,
                }],
                shape: vec![Coordinate::new(52.5, 13.4).unwrap()],
            }],
            shape: vec![Coordinate::new(52.5, 13.4).unwrap()],
        };
        let json = serde_json::to_string(&leg).unwrap();
        let back: RouteLeg = serde_json::from_str(&json).unwrap();
        assert_eq!(back.steps.len(), 1);
        assert_eq!(
            back.steps[0].maneuver_type,
            Some(ManeuverType::Roundabout(2))
        );
        assert_eq!(back.steps[0].lane_guidances.len(), 1);
        assert_eq!(back.shape.len(), 1);
    }
}
