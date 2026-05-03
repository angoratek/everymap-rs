pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::routing::{RouteOptions, RouteResponse, RouteResult, Router};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Polyline;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use types::*;

use everymap_core::domains::routing::RouteStep;

const ROUTING_BASE_URL: &str = "https://router.hereapi.com/v8";

impl From<HereRoute> for RouteResult {
    fn from(route: HereRoute) -> Self {
        let section = route.sections.into_iter().next();
        let (distance, duration, geometry, steps) = match &section {
            Some(s) => {
                let dist = s.summary.as_ref().and_then(|sum| sum.length).unwrap_or(0.0);
                let dur = s
                    .summary
                    .as_ref()
                    .and_then(|sum| sum.duration)
                    .unwrap_or(0.0);
                let geom = s
                    .polyline
                    .clone()
                    .and_then(|p| p.into_polyline_string())
                    .map(|encoded| {
                        everymap_core::types::FlexiblePolyline::decode(&encoded)
                            .map(Polyline::new)
                            .unwrap_or_else(|_| Polyline::new(vec![]))
                    })
                    .unwrap_or_else(|| Polyline::new(vec![]));
                let route_steps: Vec<RouteStep> = s
                    .turn_by_turn_actions
                    .iter()
                    .map(|a| RouteStep {
                        instruction: a.instruction.clone(),
                        distance: a.length,
                        duration: a.duration,
                        start_coordinate: None,
                        end_coordinate: None,
                    })
                    .collect();
                (dist, dur, geom, route_steps)
            }
            None => (0.0, 0.0, Polyline::new(vec![]), vec![]),
        };
        Self {
            distance,
            duration,
            geometry,
            transport_mode: None,
            steps,
            bounding_box: None,
            raw: None,
        }
    }
}

impl From<HereRouteSection> for RouteResult {
    fn from(section: HereRouteSection) -> Self {
        let distance = section
            .summary
            .as_ref()
            .and_then(|s| s.length)
            .unwrap_or(0.0);
        let duration = section
            .summary
            .as_ref()
            .and_then(|s| s.duration)
            .unwrap_or(0.0);
        let geometry = section
            .polyline
            .and_then(|p| p.into_polyline_string())
            .map(|encoded| {
                everymap_core::types::FlexiblePolyline::decode(&encoded)
                    .map(Polyline::new)
                    .unwrap_or_else(|_| Polyline::new(vec![]))
            })
            .unwrap_or_else(|| Polyline::new(vec![]));
        let steps: Vec<RouteStep> = section
            .turn_by_turn_actions
            .iter()
            .map(|a| RouteStep {
                instruction: a.instruction.clone(),
                distance: a.length,
                duration: a.duration,
                start_coordinate: None,
                end_coordinate: None,
            })
            .collect();
        Self {
            distance,
            duration,
            geometry,
            transport_mode: None,
            steps,
            bounding_box: None,
            raw: None,
        }
    }
}

/// Exhaustive options for HERE Routing API v8.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereRouteOptions {
    pub transport_mode: TransportMode,
    pub routing_mode: RoutingMode,
    pub alternatives: Option<u32>,
    pub via: Option<Vec<String>>,
    pub departure_time: Option<String>,
    pub arrival_time: Option<String>,
    pub avoid: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub units: Option<Units>,
    pub lang: Option<String>,
    pub return_fields: Option<Vec<ReturnField>>,
    pub spans: Option<Vec<String>>,
    pub vehicle: Option<Vec<String>>,
    pub consumption_model: Option<ConsumptionModel>,
    pub traffic: Option<TrafficMode>,
    pub billing_tag: Option<String>,
    pub scooter: Option<ScooterParams>,
    pub truck: Option<TruckParams>,
    pub ev: Option<EvParams>,
    pub fuel: Option<FuelParams>,
    pub driver: Option<DriverParams>,
    pub taxi: Option<TaxiParams>,
    pub tolls: Option<TollsParams>,
    pub max_speed_on_segment: Option<Vec<MaxSpeedOnSegment>>,
    pub customizations: Option<String>,
    pub currency: Option<String>,
    pub route_handle: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransportMode {
    #[default]
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Bus,
    PrivateBus,
    Scooter,
    Taxi,
    NetworkRestrictedTruck,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    #[default]
    Fast,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    Metric,
    Imperial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReturnField {
    Polyline,
    Actions,
    Instructions,
    Summary,
    TravelSummary,
    TurnByTurnActions,
    MlDuration,
    TypicalDuration,
    Elevation,
    RouteHandle,
    Passthrough,
    Incidents,
    RoutingZones,
    TruckRoadTypes,
    Tolls,
    RouteLabels,
    PotentialTimeDependentViolations,
    NoThroughRestrictions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumptionModel {
    Empirical,
    Physical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficMode {
    Live,
    Historical,
}

/// Scooter routing parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScooterParams {
    #[serde(default, rename = "allowHighway")]
    pub allow_highway: Option<bool>,
}

/// Truck routing parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TruckParams {
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub length: Option<u32>,
    #[serde(default)]
    pub weight: Option<u32>,
    #[serde(default, rename = "axleCount")]
    pub axle_count: Option<u32>,
    #[serde(default, rename = "trailerCount")]
    pub trailer_count: Option<u32>,
    #[serde(default, rename = "shippedHazardousGoods")]
    pub shipped_hazardous_goods: Option<Vec<String>>,
    #[serde(default, rename = "tunnelCategory")]
    pub tunnel_category: Option<String>,
    #[serde(default, rename = "grossCombinationWeight")]
    pub gross_combination_weight: Option<u32>,
    #[serde(default, rename = "weightPerAxle")]
    pub weight_per_axle: Option<u32>,
}

/// EV routing parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvParams {
    #[serde(default, rename = "chargingMode")]
    pub charging_mode: Option<String>,
    #[serde(default, rename = "connectorTypes")]
    pub connector_types: Option<Vec<String>>,
    #[serde(default, rename = "maxCharge")]
    pub max_charge: Option<f64>,
    #[serde(default, rename = "minCharge")]
    pub min_charge: Option<f64>,
}

/// Fuel parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FuelParams {
    #[serde(default)]
    pub free_flow_speed_table: Option<String>,
    #[serde(default)]
    pub traffic_speed_table: Option<String>,
    #[serde(default)]
    pub ascent: Option<f64>,
    #[serde(default)]
    pub descent: Option<f64>,
}

/// Driver parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriverParams {
    #[serde(default, rename = "dailyDuration")]
    pub daily_duration: Option<f64>,
    #[serde(default, rename = "breakDuration")]
    pub break_duration: Option<f64>,
    #[serde(default, rename = "restDuration")]
    pub rest_duration: Option<f64>,
}

/// Taxi parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxiParams {
    #[serde(default, rename = "allowDriveThroughTaxiRoads")]
    pub allow_drive_through_taxi_roads: Option<bool>,
}

/// Tolls parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TollsParams {
    #[serde(default)]
    pub vehicle: Option<Vec<String>>,
    #[serde(default)]
    pub transponders: Option<Vec<String>>,
}

/// Max speed on segment override.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaxSpeedOnSegment {
    #[serde(default)]
    pub segment_ref: Option<String>,
    #[serde(default)]
    pub max_speed: Option<f64>,
}

/// Implementation of Router for HERE Technologies.
pub struct HereRouter {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereRouter {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: ROUTING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core `RouteOptions` to HERE-specific `HereRouteOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn route_options_from_core(opts: &RouteOptions) -> HereRouteOptions {
    let transport_mode = match opts.transport_mode {
        Some(everymap_core::domains::routing::TransportMode::Car) | None => TransportMode::Car,
        Some(everymap_core::domains::routing::TransportMode::Truck) => TransportMode::Truck,
        Some(everymap_core::domains::routing::TransportMode::Pedestrian) => {
            TransportMode::Pedestrian
        }
        Some(everymap_core::domains::routing::TransportMode::Bicycle) => TransportMode::Bicycle,
        Some(everymap_core::domains::routing::TransportMode::Scooter) => TransportMode::Scooter,
        Some(everymap_core::domains::routing::TransportMode::Bus) => TransportMode::Bus,
        Some(everymap_core::domains::routing::TransportMode::Taxi) => TransportMode::Taxi,
        Some(everymap_core::domains::routing::TransportMode::Unknown) => TransportMode::Car,
    };

    let mut here_opts = HereRouteOptions {
        transport_mode,
        alternatives: opts.alternatives,
        departure_time: opts.departure_time.as_ref().map(|dt| dt.to_string()),
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Convert avoid types
    if !opts.avoid.is_empty() {
        here_opts.avoid = Some(
            opts.avoid
                .iter()
                .map(|a| match a {
                    everymap_core::domains::routing::AvoidType::Tolls => "tolls".to_string(),
                    everymap_core::domains::routing::AvoidType::Ferries => "ferries".to_string(),
                    everymap_core::domains::routing::AvoidType::Tunnels => "tunnels".to_string(),
                    everymap_core::domains::routing::AvoidType::Highways => "highways".to_string(),
                    everymap_core::domains::routing::AvoidType::DirtRoads => {
                        "dirtRoads".to_string()
                    }
                })
                .collect(),
        );
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("routing_mode").and_then(|v| v.as_str()) {
                here_opts.routing_mode = match v {
                    "short" => RoutingMode::Short,
                    _ => RoutingMode::Fast,
                };
            }
            if let Some(v) = obj.get("via").and_then(|v| v.as_array()) {
                here_opts.via = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("arrival_time").and_then(|v| v.as_str()) {
                here_opts.arrival_time = Some(v.to_string());
            }
            if let Some(v) = obj.get("exclude").and_then(|v| v.as_array()) {
                here_opts.exclude = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                here_opts.units =
                    serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("spans").and_then(|v| v.as_array()) {
                here_opts.spans = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("vehicle").and_then(|v| v.as_array()) {
                here_opts.vehicle = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("consumption_model").and_then(|v| v.as_str()) {
                here_opts.consumption_model =
                    serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("traffic").and_then(|v| v.as_str()) {
                here_opts.traffic =
                    serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("billing_tag").and_then(|v| v.as_str()) {
                here_opts.billing_tag = Some(v.to_string());
            }
            if let Some(v) = obj.get("scooter") {
                here_opts.scooter = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("truck") {
                here_opts.truck = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("ev") {
                here_opts.ev = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("fuel") {
                here_opts.fuel = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("driver") {
                here_opts.driver = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("taxi") {
                here_opts.taxi = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("tolls") {
                here_opts.tolls = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("max_speed_on_segment").and_then(|v| v.as_array()) {
                here_opts.max_speed_on_segment = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("customizations").and_then(|v| v.as_str()) {
                here_opts.customizations = Some(v.to_string());
            }
            if let Some(v) = obj.get("currency").and_then(|v| v.as_str()) {
                here_opts.currency = Some(v.to_string());
            }
            if let Some(v) = obj.get("route_handle").and_then(|v| v.as_str()) {
                here_opts.route_handle = Some(v.to_string());
            }
            if let Some(v) = obj.get("return_fields").and_then(|v| v.as_array()) {
                let parsed: Vec<ReturnField> = v
                    .iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect();
                if !parsed.is_empty() {
                    here_opts.return_fields = Some(parsed);
                }
            }
        }
    }

    here_opts
}

#[async_trait]
impl Router for HereRouter {
    async fn calculate_route(
        &self,
        start: &everymap_core::types::Coordinate,
        end: &everymap_core::types::Coordinate,
        options: &RouteOptions,
    ) -> EveryMapResult<RouteResponse> {
        let here_opts = route_options_from_core(options);

        let transport = match here_opts.transport_mode {
            TransportMode::Car => "car",
            TransportMode::Truck => "truck",
            TransportMode::Pedestrian => "pedestrian",
            TransportMode::Bicycle => "bicycle",
            TransportMode::Bus => "bus",
            TransportMode::PrivateBus => "privateBus",
            TransportMode::Scooter => "scooter",
            TransportMode::Taxi => "taxi",
            TransportMode::NetworkRestrictedTruck => "networkRestrictedTruck",
        };

        let mode = match here_opts.routing_mode {
            RoutingMode::Fast => "fast",
            RoutingMode::Short => "short",
        };

        let mut params: Vec<(String, String)> = vec![
            ("transportMode".to_string(), transport.to_string()),
            ("routingMode".to_string(), mode.to_string()),
            ("origin".to_string(), format!("{},{}", start.lat, start.lng)),
            ("destination".to_string(), format!("{},{}", end.lat, end.lng)),
        ];

        if let Some(return_fields) = &here_opts.return_fields {
            params.push((
                "return".to_string(),
                return_fields
                    .iter()
                    .map(crate::util::enum_as_str)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        } else {
            params.push(("return".to_string(), "polyline,summary".to_string()));
        }

        if let Some(alternatives) = here_opts.alternatives {
            params.push(("alternatives".to_string(), alternatives.to_string()));
        }
        if let Some(via) = &here_opts.via {
            for v in via {
                params.push(("via".to_string(), v.clone()));
            }
        }
        if let Some(departure_time) = &here_opts.departure_time {
            params.push(("departureTime".to_string(), departure_time.clone()));
        }
        if let Some(at) = &here_opts.arrival_time {
            params.push(("arrivalTime".to_string(), at.clone()));
        }
        if let Some(avoid) = &here_opts.avoid {
            params.push(("avoid".to_string(), avoid.join(",")));
        }
        if let Some(exclude) = &here_opts.exclude {
            params.push(("exclude".to_string(), exclude.join(",")));
        }
        if let Some(units) = &here_opts.units {
            params.push(("units".to_string(), crate::util::enum_as_str(units)));
        }
        if let Some(lang) = &here_opts.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(spans) = &here_opts.spans {
            params.push(("spans".to_string(), spans.join(",")));
        }
        if let Some(vehicle) = &here_opts.vehicle {
            params.push(("vehicle".to_string(), vehicle.join(",")));
        }
        if let Some(consumption_model) = &here_opts.consumption_model {
            params.push((
                "consumptionModel".to_string(),
                crate::util::enum_as_str(consumption_model),
            ));
        }
        // Scooter options
        if let Some(scooter) = &here_opts.scooter {
            if let Some(allow_highway) = scooter.allow_highway {
                params.push(("scooter[allowHighway]".to_string(), allow_highway.to_string()));
            }
        }
        // Truck options
        if let Some(truck) = &here_opts.truck {
            if let Some(height) = truck.height {
                params.push(("truck[height]".to_string(), format!("{}m", height)));
            }
            if let Some(width) = truck.width {
                params.push(("truck[width]".to_string(), format!("{}m", width)));
            }
            if let Some(length) = truck.length {
                params.push(("truck[length]".to_string(), format!("{}m", length)));
            }
            if let Some(weight) = truck.weight {
                params.push(("truck[grossWeight]".to_string(), format!("{}kg", weight)));
            }
            if let Some(axle_count) = truck.axle_count {
                params.push(("truck[axleCount]".to_string(), axle_count.to_string()));
            }
            if let Some(trailer_count) = truck.trailer_count {
                params.push(("truck[trailerCount]".to_string(), trailer_count.to_string()));
            }
            if let Some(hazardous_goods) = &truck.shipped_hazardous_goods {
                params.push(("truck[shippedHazardousGoods]".to_string(), hazardous_goods.join(",")));
            }
            if let Some(tunnel_category) = &truck.tunnel_category {
                params.push(("truck[tunnelCategory]".to_string(), tunnel_category.clone()));
            }
            if let Some(gcw) = truck.gross_combination_weight {
                params.push(("truck[grossCombinationWeight]".to_string(), format!("{}kg", gcw)));
            }
            if let Some(weight_per_axle) = truck.weight_per_axle {
                params.push(("truck[weightPerAxle]".to_string(), format!("{}kg", weight_per_axle)));
            }
        }
        // EV options
        if let Some(ev) = &here_opts.ev {
            if let Some(charging_mode) = &ev.charging_mode {
                params.push(("ev[chargingMode]".to_string(), charging_mode.clone()));
            }
            if let Some(connector_types) = &ev.connector_types {
                params.push(("ev[connectorTypes]".to_string(), connector_types.join(",")));
            }
            if let Some(max_charge) = ev.max_charge {
                params.push(("ev[maxCharge]".to_string(), max_charge.to_string()));
            }
            if let Some(min_charge) = ev.min_charge {
                params.push(("ev[minCharge]".to_string(), min_charge.to_string()));
            }
        }
        // Fuel options
        if let Some(fuel) = &here_opts.fuel {
            if let Some(free_flow_speed_table) = &fuel.free_flow_speed_table {
                params.push(("fuel[freeFlowSpeedTable]".to_string(), free_flow_speed_table.clone()));
            }
            if let Some(traffic_speed_table) = &fuel.traffic_speed_table {
                params.push(("fuel[trafficSpeedTable]".to_string(), traffic_speed_table.clone()));
            }
            if let Some(ascent) = fuel.ascent {
                params.push(("fuel[ascent]".to_string(), ascent.to_string()));
            }
            if let Some(descent) = fuel.descent {
                params.push(("fuel[descent]".to_string(), descent.to_string()));
            }
        }
        // Driver options
        if let Some(driver) = &here_opts.driver {
            if let Some(daily_duration) = driver.daily_duration {
                params.push(("driver[dailyDuration]".to_string(), daily_duration.to_string()));
            }
            if let Some(break_duration) = driver.break_duration {
                params.push(("driver[breakDuration]".to_string(), break_duration.to_string()));
            }
            if let Some(rest_duration) = driver.rest_duration {
                params.push(("driver[restDuration]".to_string(), rest_duration.to_string()));
            }
        }
        // Taxi options
        if let Some(taxi) = &here_opts.taxi {
            if let Some(allow_drive_through) = taxi.allow_drive_through_taxi_roads {
                params.push(("taxi[allowDriveThroughTaxiRoads]".to_string(), allow_drive_through.to_string()));
            }
        }
        // Tolls options
        if let Some(tolls) = &here_opts.tolls {
            if let Some(vehicle) = &tolls.vehicle {
                params.push(("tolls[vehicle]".to_string(), vehicle.join(",")));
            }
            if let Some(transponders) = &tolls.transponders {
                params.push(("tolls[transponders]".to_string(), transponders.join(",")));
            }
        }
        // Max speed on segment overrides
        if let Some(segments) = &here_opts.max_speed_on_segment {
            for (i, segment) in segments.iter().enumerate() {
                if let Some(segment_ref) = &segment.segment_ref {
                    params.push((format!("maxSpeedOnSegment[{}][segmentRef]", i), segment_ref.clone()));
                }
                if let Some(max_speed) = segment.max_speed {
                    params.push((format!("maxSpeedOnSegment[{}][maxSpeed]", i), max_speed.to_string()));
                }
            }
        }
        if let Some(traffic) = &here_opts.traffic {
            params.push(("traffic".to_string(), crate::util::enum_as_str(traffic)));
        }
        if let Some(billing_tag) = &here_opts.billing_tag {
            params.push(("billingTag".to_string(), billing_tag.clone()));
        }
        if let Some(currency) = &here_opts.currency {
            params.push(("currency".to_string(), currency.clone()));
        }
        if let Some(customizations) = &here_opts.customizations {
            params.push(("customizations".to_string(), customizations.clone()));
        }
        if let Some(route_handle) = &here_opts.route_handle {
            params.push(("routeHandle".to_string(), route_handle.clone()));
        }

        let url = format!("{}/routes", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereRouteApiResponse = self.client.request_json(builder).await?;

        let routes: Vec<RouteResult> = here_res.routes.into_iter().map(RouteResult::from).collect();

        Ok(RouteResponse { routes })
    }
}
