pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::matching::{
    MatchedPoint, MatchingOptions, RouteMatcher, TraceResponse,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use types::*;

const MATCHING_BASE_URL: &str = "https://routematching.hereapi.com/v8";

/// Exhaustive options for HERE Route Matching API v8.
/// Organized into groups matching the API parameter structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereMatchingOptions {
    // --- Match mode ---
    pub route_match: Option<u8>,
    pub mode: Option<MatchMode>,
    pub routing_mode: Option<MatchRoutingMode>,
    pub traffic: Option<MatchTrafficMode>,
    pub legal: Option<LegalConstraint>,
    pub traverse_gates: Option<bool>,
    pub oneway: Option<bool>,

    // --- Matching parameters ---
    pub map_match_radius: Option<u32>,
    pub align_to_gps_time: Option<bool>,
    pub ignore_zero_speed_points: Option<bool>,
    pub wp_dist: Option<u32>,
    pub speed_fc_cat: Option<String>,
    pub map_match_tolerance: Option<u32>,
    pub heading: Option<f64>,

    // --- Vehicle dimensions & weight ---
    pub limited_weight: Option<u32>,
    pub height: Option<u32>,
    pub length: Option<u32>,
    pub width: Option<u32>,
    pub vehicle_number_axles: Option<u32>,
    pub trailer_number_axles: Option<u32>,
    pub trailer_type: Option<TrailerType>,
    pub vehicle_weight: Option<u32>,
    pub trailer_weight: Option<u32>,
    pub weight_per_axle: Option<u32>,
    pub height_above_1st_axle: Option<u32>,
    pub trailers_count: Option<u32>,

    // --- Emission & fuel ---
    pub emission_type: Option<EmissionType>,
    pub co2_emission_class: Option<u32>,
    pub fuel_type: Option<MatchFuelType>,
    pub hybrid: Option<bool>,

    // --- Restrictions ---
    pub avoid_links: Option<Vec<String>>,
    pub avoid_areas: Option<Vec<String>>,
    pub avoid_turns: Option<Vec<String>>,
    pub avoid_features: Option<Vec<AvoidFeature>>,
    pub avoid_private: Option<bool>,
    pub avoid_country_change: Option<bool>,
    pub shipped_hazardous_goods: Option<Vec<HazardousGoodsType>>,
    pub tunnel_category: Option<TunnelCategory>,

    // --- Commercial ---
    pub commercial: Option<bool>,
    pub passengers_count: Option<u32>,
    pub tires_count: Option<u32>,
    pub disabled_equipped: Option<bool>,
    pub license_plate: Option<String>,

    // --- Time ---
    pub departure: Option<String>,
    pub arrival: Option<String>,

    // --- Response attributes ---
    pub leg_attributes: Option<String>,
    pub link_attributes: Option<String>,
    pub response_attributes: Option<String>,
    pub route_attributes: Option<String>,
    pub meta_attributes: Option<String>,
    pub maneuver_attributes: Option<String>,
    pub instruction_format: Option<InstructionFormat>,
    pub language: Option<String>,

    // --- Toll ---
    pub toll_vehicle_type: Option<u32>,
    pub toll_pass: Option<String>,
    pub currency: Option<String>,
    pub driver_cost: Option<String>,
    pub vehicle_cost: Option<String>,
    pub vehicle_cost_on_ferry: Option<String>,
    pub cost_per_consumption_unit: Option<String>,

    // --- Advanced ---
    pub max_speed: Option<u32>,
    pub alternatives: Option<u32>,
    pub truck_verified: Option<bool>,
    pub ignore_waypoint_vehicle_restriction: Option<bool>,
    pub admin_truck_restrictions: Option<bool>,
    pub ignore_preferred_routes: Option<bool>,
    pub exclude_zone_types: Option<String>,
    pub overlays: Option<String>,
    pub custom_restrict_limit: Option<String>,
    pub custom_attributes: Option<String>,
    pub custom_consumption_details: Option<String>,
    pub timeout: Option<u32>,
    pub driving_report: Option<bool>,
    pub ehorizon_limits: Option<String>,
}

/// Implementation of RouteMatcher for HERE Technologies.
pub struct HereRouteMatcher {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereRouteMatcher {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: MATCHING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

// Internal deserialization for the real API response format
// The HERE Route Matching API v8 returns:
// {"response": {"route": [{"waypoint": [...], "leg": [...], "mode": {...}}]}}
#[derive(Debug, Deserialize)]
struct HereMatchApiResponseWrapper {
    #[serde(default)]
    response: HereMatchApiResponseInner,
}

#[derive(Debug, Default, Deserialize)]
struct HereMatchApiResponseInner {
    #[serde(default)]
    route: Vec<HereMatchApiRoute>,
}

#[derive(Debug, Default, Deserialize)]
struct HereMatchApiRoute {
    #[serde(default)]
    waypoint: Vec<HereMatchApiWaypoint>,
    #[serde(default)]
    leg: Vec<HereMatchApiLeg>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct HereMatchApiWaypoint {
    #[serde(default, rename = "mappedPosition")]
    mapped_position: Option<HereMatchApiPosition>,
    #[serde(default, rename = "originalPosition")]
    original_position: Option<HereMatchApiPosition>,
    #[serde(default, rename = "confidenceValue")]
    confidence_value: Option<f64>,
    #[serde(default, rename = "matchDistance")]
    match_distance: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct HereMatchApiPosition {
    #[serde(default)]
    latitude: f64,
    #[serde(default)]
    longitude: f64,
}

#[derive(Debug, Default, Deserialize)]
struct HereMatchApiLeg {
    #[serde(default)]
    length: f64,
    #[serde(default, rename = "travelTime")]
    travel_time: f64,
}

impl From<HereMatchedPoint> for MatchedPoint {
    fn from(p: HereMatchedPoint) -> Self {
        Self {
            coordinate: Coordinate::new(p.lat.unwrap_or(0.0), p.lng.unwrap_or(0.0))
                .unwrap_or(Coordinate::ORIGIN),
            confidence: p.point_match_probability,
            road_name: None,
        }
    }
}

fn add_option(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<impl std::fmt::Display>,
) {
    if let Some(v) = value {
        params.push((key.to_string(), v.to_string()));
    }
}

fn add_option_ref(params: &mut Vec<(String, String)>, key: &str, value: Option<&String>) {
    if let Some(v) = value {
        params.push((key.to_string(), v.clone()));
    }
}

/// Convert core `MatchingOptions` to HERE-specific `HereMatchingOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn matching_options_from_core(opts: &MatchingOptions) -> HereMatchingOptions {
    let mut here_opts = HereMatchingOptions {
        heading: opts.heading,
        departure: opts.departure_time.as_ref().map(|dt| dt.to_string()),
        ..Default::default()
    };

    // Convert avoid types
    if !opts.avoid.is_empty() {
        here_opts.avoid_features = Some(
            opts.avoid
                .iter()
                .map(|a| match a {
                    everymap_core::domains::routing::AvoidType::Tolls => AvoidFeature::TollRoad,
                    everymap_core::domains::routing::AvoidType::Ferries => AvoidFeature::Ferry,
                    everymap_core::domains::routing::AvoidType::Tunnels => AvoidFeature::Tunnel,
                    everymap_core::domains::routing::AvoidType::Highways => {
                        AvoidFeature::ControlledAccessHighway
                    }
                    everymap_core::domains::routing::AvoidType::DirtRoads => AvoidFeature::DirtRoad,
                })
                .collect(),
        );
    }

    // Convert core transport_mode to HERE match mode (provider_extra can still override)
    if let Some(transport_mode) = &opts.transport_mode {
        here_opts.mode = match transport_mode {
            everymap_core::domains::routing::TransportMode::Car => Some(MatchMode::Car),
            everymap_core::domains::routing::TransportMode::Truck => Some(MatchMode::Truck),
            everymap_core::domains::routing::TransportMode::Pedestrian => {
                Some(MatchMode::Pedestrian)
            }
            everymap_core::domains::routing::TransportMode::Bicycle => Some(MatchMode::Bicycle),
            everymap_core::domains::routing::TransportMode::Bus => Some(MatchMode::Bus),
            everymap_core::domains::routing::TransportMode::Scooter => {
                Some(MatchMode::Motorcycle)
            }
            everymap_core::domains::routing::TransportMode::Taxi => Some(MatchMode::Taxi),
            _ => Some(MatchMode::Car),
        };
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("route_match").and_then(|v| v.as_u64()) {
                here_opts.route_match = Some(v as u8);
            }
            if let Some(v) = obj.get("mode") {
                here_opts.mode = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("routing_mode") {
                here_opts.routing_mode = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("traffic") {
                here_opts.traffic = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("legal") {
                here_opts.legal = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("traverse_gates").and_then(|v| v.as_bool()) {
                here_opts.traverse_gates = Some(v);
            }
            if let Some(v) = obj.get("oneway").and_then(|v| v.as_bool()) {
                here_opts.oneway = Some(v);
            }
            if let Some(v) = obj.get("map_match_radius").and_then(|v| v.as_u64()) {
                here_opts.map_match_radius = Some(v as u32);
            }
            if let Some(v) = obj.get("align_to_gps_time").and_then(|v| v.as_bool()) {
                here_opts.align_to_gps_time = Some(v);
            }
            if let Some(v) = obj
                .get("ignore_zero_speed_points")
                .and_then(|v| v.as_bool())
            {
                here_opts.ignore_zero_speed_points = Some(v);
            }
            if let Some(v) = obj.get("wp_dist").and_then(|v| v.as_u64()) {
                here_opts.wp_dist = Some(v as u32);
            }
            if let Some(v) = obj.get("speed_fc_cat").and_then(|v| v.as_str()) {
                here_opts.speed_fc_cat = Some(v.to_string());
            }
            if let Some(v) = obj.get("map_match_tolerance").and_then(|v| v.as_u64()) {
                here_opts.map_match_tolerance = Some(v as u32);
            }
            if let Some(v) = obj.get("limited_weight").and_then(|v| v.as_u64()) {
                here_opts.limited_weight = Some(v as u32);
            }
            if let Some(v) = obj.get("height").and_then(|v| v.as_u64()) {
                here_opts.height = Some(v as u32);
            }
            if let Some(v) = obj.get("length").and_then(|v| v.as_u64()) {
                here_opts.length = Some(v as u32);
            }
            if let Some(v) = obj.get("width").and_then(|v| v.as_u64()) {
                here_opts.width = Some(v as u32);
            }
            if let Some(v) = obj.get("vehicle_number_axles").and_then(|v| v.as_u64()) {
                here_opts.vehicle_number_axles = Some(v as u32);
            }
            if let Some(v) = obj.get("trailer_number_axles").and_then(|v| v.as_u64()) {
                here_opts.trailer_number_axles = Some(v as u32);
            }
            if let Some(v) = obj.get("trailer_type") {
                here_opts.trailer_type = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("vehicle_weight").and_then(|v| v.as_u64()) {
                here_opts.vehicle_weight = Some(v as u32);
            }
            if let Some(v) = obj.get("trailer_weight").and_then(|v| v.as_u64()) {
                here_opts.trailer_weight = Some(v as u32);
            }
            if let Some(v) = obj.get("weight_per_axle").and_then(|v| v.as_u64()) {
                here_opts.weight_per_axle = Some(v as u32);
            }
            if let Some(v) = obj.get("height_above_1st_axle").and_then(|v| v.as_u64()) {
                here_opts.height_above_1st_axle = Some(v as u32);
            }
            if let Some(v) = obj.get("trailers_count").and_then(|v| v.as_u64()) {
                here_opts.trailers_count = Some(v as u32);
            }
            if let Some(v) = obj.get("emission_type") {
                here_opts.emission_type = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("co2_emission_class").and_then(|v| v.as_u64()) {
                here_opts.co2_emission_class = Some(v as u32);
            }
            if let Some(v) = obj.get("fuel_type") {
                here_opts.fuel_type = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("hybrid").and_then(|v| v.as_bool()) {
                here_opts.hybrid = Some(v);
            }
            if let Some(v) = obj.get("avoid_links").and_then(|v| v.as_array()) {
                here_opts.avoid_links = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("avoid_areas").and_then(|v| v.as_array()) {
                here_opts.avoid_areas = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("avoid_turns").and_then(|v| v.as_array()) {
                here_opts.avoid_turns = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("avoid_private").and_then(|v| v.as_bool()) {
                here_opts.avoid_private = Some(v);
            }
            if let Some(v) = obj.get("avoid_country_change").and_then(|v| v.as_bool()) {
                here_opts.avoid_country_change = Some(v);
            }
            if let Some(v) = obj
                .get("shipped_hazardous_goods")
                .and_then(|v| v.as_array())
            {
                here_opts.shipped_hazardous_goods = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("tunnel_category") {
                here_opts.tunnel_category = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("commercial").and_then(|v| v.as_bool()) {
                here_opts.commercial = Some(v);
            }
            if let Some(v) = obj.get("passengers_count").and_then(|v| v.as_u64()) {
                here_opts.passengers_count = Some(v as u32);
            }
            if let Some(v) = obj.get("tires_count").and_then(|v| v.as_u64()) {
                here_opts.tires_count = Some(v as u32);
            }
            if let Some(v) = obj.get("disabled_equipped").and_then(|v| v.as_bool()) {
                here_opts.disabled_equipped = Some(v);
            }
            if let Some(v) = obj.get("license_plate").and_then(|v| v.as_str()) {
                here_opts.license_plate = Some(v.to_string());
            }
            if let Some(v) = obj.get("arrival").and_then(|v| v.as_str()) {
                here_opts.arrival = Some(v.to_string());
            }
            if let Some(v) = obj.get("leg_attributes").and_then(|v| v.as_str()) {
                here_opts.leg_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("link_attributes").and_then(|v| v.as_str()) {
                here_opts.link_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("response_attributes").and_then(|v| v.as_str()) {
                here_opts.response_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("route_attributes").and_then(|v| v.as_str()) {
                here_opts.route_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("meta_attributes").and_then(|v| v.as_str()) {
                here_opts.meta_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("maneuver_attributes").and_then(|v| v.as_str()) {
                here_opts.maneuver_attributes = Some(v.to_string());
            }
            if let Some(v) = obj.get("instruction_format") {
                here_opts.instruction_format = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("toll_vehicle_type").and_then(|v| v.as_u64()) {
                here_opts.toll_vehicle_type = Some(v as u32);
            }
            if let Some(v) = obj.get("toll_pass").and_then(|v| v.as_str()) {
                here_opts.toll_pass = Some(v.to_string());
            }
            if let Some(v) = obj.get("currency").and_then(|v| v.as_str()) {
                here_opts.currency = Some(v.to_string());
            }
            if let Some(v) = obj.get("driver_cost").and_then(|v| v.as_str()) {
                here_opts.driver_cost = Some(v.to_string());
            }
            if let Some(v) = obj.get("vehicle_cost").and_then(|v| v.as_str()) {
                here_opts.vehicle_cost = Some(v.to_string());
            }
            if let Some(v) = obj.get("vehicle_cost_on_ferry").and_then(|v| v.as_str()) {
                here_opts.vehicle_cost_on_ferry = Some(v.to_string());
            }
            if let Some(v) = obj
                .get("cost_per_consumption_unit")
                .and_then(|v| v.as_str())
            {
                here_opts.cost_per_consumption_unit = Some(v.to_string());
            }
            if let Some(v) = obj.get("max_speed").and_then(|v| v.as_u64()) {
                here_opts.max_speed = Some(v as u32);
            }
            if let Some(v) = obj.get("alternatives").and_then(|v| v.as_u64()) {
                here_opts.alternatives = Some(v as u32);
            }
            if let Some(v) = obj.get("truck_verified").and_then(|v| v.as_bool()) {
                here_opts.truck_verified = Some(v);
            }
            if let Some(v) = obj
                .get("ignore_waypoint_vehicle_restriction")
                .and_then(|v| v.as_bool())
            {
                here_opts.ignore_waypoint_vehicle_restriction = Some(v);
            }
            if let Some(v) = obj
                .get("admin_truck_restrictions")
                .and_then(|v| v.as_bool())
            {
                here_opts.admin_truck_restrictions = Some(v);
            }
            if let Some(v) = obj.get("ignore_preferred_routes").and_then(|v| v.as_bool()) {
                here_opts.ignore_preferred_routes = Some(v);
            }
            if let Some(v) = obj.get("exclude_zone_types").and_then(|v| v.as_str()) {
                here_opts.exclude_zone_types = Some(v.to_string());
            }
            if let Some(v) = obj.get("overlays").and_then(|v| v.as_str()) {
                here_opts.overlays = Some(v.to_string());
            }
            if let Some(v) = obj.get("custom_restrict_limit").and_then(|v| v.as_str()) {
                here_opts.custom_restrict_limit = Some(v.to_string());
            }
            if let Some(v) = obj.get("custom_attributes").and_then(|v| v.as_str()) {
                here_opts.custom_attributes = Some(v.to_string());
            }
            if let Some(v) = obj
                .get("custom_consumption_details")
                .and_then(|v| v.as_str())
            {
                here_opts.custom_consumption_details = Some(v.to_string());
            }
            if let Some(v) = obj.get("timeout").and_then(|v| v.as_u64()) {
                here_opts.timeout = Some(v as u32);
            }
            if let Some(v) = obj.get("driving_report").and_then(|v| v.as_bool()) {
                here_opts.driving_report = Some(v);
            }
            if let Some(v) = obj.get("ehorizon_limits").and_then(|v| v.as_str()) {
                here_opts.ehorizon_limits = Some(v.to_string());
            }
            // Convert transport_mode from core
            if let Some(v) = obj.get("transport_mode") {
                if let Ok(transport_mode) = serde_json::from_value::<
                    everymap_core::domains::routing::TransportMode,
                >(v.clone())
                {
                    here_opts.mode = match transport_mode {
                        everymap_core::domains::routing::TransportMode::Car => Some(MatchMode::Car),
                        everymap_core::domains::routing::TransportMode::Truck => {
                            Some(MatchMode::Truck)
                        }
                        everymap_core::domains::routing::TransportMode::Pedestrian => {
                            Some(MatchMode::Pedestrian)
                        }
                        everymap_core::domains::routing::TransportMode::Bicycle => {
                            Some(MatchMode::Bicycle)
                        }
                        everymap_core::domains::routing::TransportMode::Bus => Some(MatchMode::Bus),
                        everymap_core::domains::routing::TransportMode::Scooter => {
                            Some(MatchMode::Motorcycle)
                        }
                        everymap_core::domains::routing::TransportMode::Taxi => {
                            Some(MatchMode::Taxi)
                        }
                        everymap_core::domains::routing::TransportMode::Unknown => {
                            Some(MatchMode::Car)
                        }
                    };
                }
            }
        }
    }

    // Default to car mode if none specified
    if here_opts.mode.is_none() {
        here_opts.mode = Some(MatchMode::Car);
    }
    // Default routing mode to fastest if none specified
    if here_opts.routing_mode.is_none() {
        here_opts.routing_mode = Some(MatchRoutingMode::Fastest);
    }
    // Default traffic to disabled if none specified
    if here_opts.traffic.is_none() {
        here_opts.traffic = Some(MatchTrafficMode::Disabled);
    }

    here_opts
}

#[async_trait]
impl RouteMatcher for HereRouteMatcher {
    async fn match_route(
        &self,
        points: &[Coordinate],
        options: &MatchingOptions,
    ) -> EveryMapResult<TraceResponse> {
        let opts = matching_options_from_core(options);

        // Use waypointN format: waypoint0=lat,lng&waypoint1=lat,lng&...
        let mut params: Vec<(String, String)> = vec![];
        for (i, p) in points.iter().enumerate() {
            params.push((format!("waypoint{}", i), format!("{},{}", p.lat, p.lng)));
        }

        // Default to routeMatch=1 if not specified
        if opts.route_match.is_none() {
            params.push(("routeMatch".to_string(), "1".to_string()));
        }

        let opts = &opts;

        // Match mode — compound format: {routing_type};{transport_mode}[;traffic:{enabled|disabled}]
        add_option(&mut params, "routeMatch", opts.route_match);
        {
            let routing = match &opts.routing_mode {
                Some(MatchRoutingMode::Shortest) => "shortest",
                _ => "fastest",
            };
            let transport = match &opts.mode {
                Some(m) => crate::util::enum_as_str(m),
                None => "car".to_string(),
            };
            let traffic = match &opts.traffic {
                Some(MatchTrafficMode::Enabled) => ";traffic:enabled",
                _ => ";traffic:disabled",
            };
            params.push(("mode".to_string(), format!("{};{}{}", routing, transport, traffic)));
        }
        if let Some(legal) = &opts.legal {
            params.push(("legal".to_string(), crate::util::enum_as_str(legal)));
        }
        add_option(&mut params, "traverseGates", opts.traverse_gates);
        add_option(&mut params, "oneway", opts.oneway);

        // Matching parameters
        add_option(&mut params, "mapMatchRadius", opts.map_match_radius);
        add_option(&mut params, "alignToGpsTime", opts.align_to_gps_time);
        add_option(
            &mut params,
            "ignoreZeroSpeedPoints",
            opts.ignore_zero_speed_points,
        );
        add_option(&mut params, "wpDist", opts.wp_dist);
        add_option_ref(&mut params, "speedFcCat", opts.speed_fc_cat.as_ref());
        add_option(&mut params, "mapMatchTolerance", opts.map_match_tolerance);
        add_option(&mut params, "heading", opts.heading);

        // Vehicle dimensions
        add_option(&mut params, "limitedWeight", opts.limited_weight);
        add_option(&mut params, "height", opts.height);
        add_option(&mut params, "length", opts.length);
        add_option(&mut params, "width", opts.width);
        add_option(&mut params, "vehicleNumberAxles", opts.vehicle_number_axles);
        add_option(&mut params, "trailerNumberAxles", opts.trailer_number_axles);
        if let Some(tt) = &opts.trailer_type {
            params.push(("trailerType".to_string(), crate::util::enum_as_str(tt)));
        }
        add_option(&mut params, "vehicleWeight", opts.vehicle_weight);
        add_option(&mut params, "trailerWeight", opts.trailer_weight);
        add_option(&mut params, "weightPerAxle", opts.weight_per_axle);
        add_option(
            &mut params,
            "heightAbove1stAxle",
            opts.height_above_1st_axle,
        );
        add_option(&mut params, "trailersCount", opts.trailers_count);

        // Emission & fuel
        if let Some(et) = &opts.emission_type {
            params.push(("emissionType".to_string(), crate::util::enum_as_str(et)));
        }
        add_option(&mut params, "co2EmissionClass", opts.co2_emission_class);
        if let Some(ft) = &opts.fuel_type {
            params.push(("fuelType".to_string(), crate::util::enum_as_str(ft)));
        }
        add_option(&mut params, "hybrid", opts.hybrid);

        // Restrictions
        if let Some(links) = &opts.avoid_links {
            params.push(("avoidLinks".to_string(), links.join(",")));
        }
        if let Some(areas) = &opts.avoid_areas {
            params.push(("avoidAreas".to_string(), areas.join(",")));
        }
        if let Some(turns) = &opts.avoid_turns {
            params.push(("avoidTurns".to_string(), turns.join(",")));
        }
        if let Some(af) = &opts.avoid_features {
            params.push((
                "avoidFeatures".to_string(),
                af.iter()
                    .map(crate::util::enum_as_str)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        add_option(&mut params, "avoidPrivate", opts.avoid_private);
        add_option(&mut params, "avoidCountryChange", opts.avoid_country_change);
        if let Some(hg) = &opts.shipped_hazardous_goods {
            params.push((
                "shippedHazardousGoods".to_string(),
                hg.iter()
                    .map(crate::util::enum_as_str)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if let Some(tc) = &opts.tunnel_category {
            params.push(("tunnelCategory".to_string(), crate::util::enum_as_str(tc)));
        }

        // Commercial
        add_option(&mut params, "commercial", opts.commercial);
        add_option(&mut params, "passengersCount", opts.passengers_count);
        add_option(&mut params, "tiresCount", opts.tires_count);
        add_option(&mut params, "disabledEquipped", opts.disabled_equipped);
        add_option_ref(&mut params, "licensePlate", opts.license_plate.as_ref());

        // Time
        add_option_ref(&mut params, "departure", opts.departure.as_ref());
        add_option_ref(&mut params, "arrival", opts.arrival.as_ref());

        // Response attributes
        add_option_ref(&mut params, "legAttributes", opts.leg_attributes.as_ref());
        add_option_ref(&mut params, "linkAttributes", opts.link_attributes.as_ref());
        add_option_ref(
            &mut params,
            "responseAttributes",
            opts.response_attributes.as_ref(),
        );
        add_option_ref(
            &mut params,
            "routeAttributes",
            opts.route_attributes.as_ref(),
        );
        add_option_ref(&mut params, "metaAttributes", opts.meta_attributes.as_ref());
        add_option_ref(
            &mut params,
            "maneuverAttributes",
            opts.maneuver_attributes.as_ref(),
        );
        if let Some(fmt) = &opts.instruction_format {
            params.push((
                "instructionFormat".to_string(),
                crate::util::enum_as_str(fmt),
            ));
        }
        add_option_ref(&mut params, "language", opts.language.as_ref());

        // Toll
        add_option(&mut params, "tollVehicleType", opts.toll_vehicle_type);
        add_option_ref(&mut params, "tollPass", opts.toll_pass.as_ref());
        add_option_ref(&mut params, "currency", opts.currency.as_ref());
        add_option_ref(&mut params, "driverCost", opts.driver_cost.as_ref());
        add_option_ref(&mut params, "vehicleCost", opts.vehicle_cost.as_ref());
        add_option_ref(
            &mut params,
            "vehicleCostOnFerry",
            opts.vehicle_cost_on_ferry.as_ref(),
        );
        add_option_ref(
            &mut params,
            "costPerConsumptionUnit",
            opts.cost_per_consumption_unit.as_ref(),
        );

        // Advanced
        add_option(&mut params, "maxSpeed", opts.max_speed);
        add_option(&mut params, "alternatives", opts.alternatives);
        add_option(&mut params, "truckVerified", opts.truck_verified);
        add_option(
            &mut params,
            "ignoreWaypointVehicleRestriction",
            opts.ignore_waypoint_vehicle_restriction,
        );
        add_option(
            &mut params,
            "adminTruckRestrictions",
            opts.admin_truck_restrictions,
        );
        add_option(
            &mut params,
            "ignorePreferredRoutes",
            opts.ignore_preferred_routes,
        );
        add_option_ref(
            &mut params,
            "excludeZoneTypes",
            opts.exclude_zone_types.as_ref(),
        );
        add_option_ref(&mut params, "overlays", opts.overlays.as_ref());
        add_option_ref(
            &mut params,
            "customRestrLimit",
            opts.custom_restrict_limit.as_ref(),
        );
        add_option_ref(
            &mut params,
            "customAttributes",
            opts.custom_attributes.as_ref(),
        );
        add_option_ref(
            &mut params,
            "customConsumptionDetails",
            opts.custom_consumption_details.as_ref(),
        );
        add_option(&mut params, "timeout", opts.timeout);
        add_option(&mut params, "drivingReport", opts.driving_report);
        add_option_ref(&mut params, "ehorizonLimits", opts.ehorizon_limits.as_ref());

        let url = format!("{}/match/routelinks", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereMatchApiResponseWrapper = self.client.request_json(builder).await?;

        // Extract matched points and total distance from the API response
        let route = here_res.response.route.into_iter().next();
        let (matched_points, distance, duration) = match route {
            Some(r) => {
                let points = r
                    .waypoint
                    .into_iter()
                    .filter_map(|wp| {
                        let position = wp.mapped_position.or(wp.original_position)?;
                        Some(MatchedPoint {
                            coordinate: Coordinate::new(position.latitude, position.longitude)
                                .unwrap_or(Coordinate::ORIGIN),
                            confidence: wp.confidence_value,
                            road_name: None,
                        })
                    })
                    .collect();
                let total_length: f64 = r.leg.iter().map(|l| l.length).sum();
                let total_time: f64 = r.leg.iter().map(|l| l.travel_time).sum();
                (
                    points,
                    total_length,
                    if total_time > 0.0 {
                        Some(total_time)
                    } else {
                        None
                    },
                )
            }
            None => (vec![], 0.0, None),
        };

        Ok(TraceResponse {
            matched_points,
            distance,
            duration,
            raw: None,
        })
    }
}
