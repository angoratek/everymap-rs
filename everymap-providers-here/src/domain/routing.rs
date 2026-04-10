pub mod types;

use async_trait::async_trait;
use everymap_core::domains::routing::{Router, RouteOptions, RouteResponse, RouteResult};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Polyline;
use crate::client::HereClient;
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
                let dist = s.summary.length.unwrap_or(0.0);
                let dur = s.summary.duration.unwrap_or(0.0);
                let geom = s.polyline.as_ref()
                    .and_then(|p| p.polyline.as_ref())
                    .map(|encoded| {
                        everymap_core::types::FlexiblePolyline::decode(encoded)
                            .map(Polyline::new)
                            .unwrap_or_else(|_| Polyline::new(vec![]))
                    })
                    .unwrap_or_else(|| Polyline::new(vec![]));
                let route_steps: Vec<RouteStep> = s.turn_by_turn_actions.iter().map(|a| {
                    RouteStep {
                        instruction: a.instruction.clone(),
                        distance: a.length,
                        duration: a.duration,
                        start_coordinate: None,
                        end_coordinate: None,
                    }
                }).collect();
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
        let distance = section.summary.length.unwrap_or(0.0);
        let duration = section.summary.duration.unwrap_or(0.0);
        let geometry = section.polyline.as_ref()
            .and_then(|p| p.polyline.as_ref())
            .map(|encoded| {
                everymap_core::types::FlexiblePolyline::decode(encoded)
                    .map(Polyline::new)
                    .unwrap_or_else(|_| Polyline::new(vec![]))
            })
            .unwrap_or_else(|| Polyline::new(vec![]));
        let steps: Vec<RouteStep> = section.turn_by_turn_actions.iter().map(|a| {
            RouteStep {
                instruction: a.instruction.clone(),
                distance: a.length,
                duration: a.duration,
                start_coordinate: None,
                end_coordinate: None,
            }
        }).collect();
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
    client: Arc<HereClient>,
    base_url: String,
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
        Some(everymap_core::domains::routing::TransportMode::Pedestrian) => TransportMode::Pedestrian,
        Some(everymap_core::domains::routing::TransportMode::Bicycle) => TransportMode::Bicycle,
        Some(everymap_core::domains::routing::TransportMode::Scooter) => TransportMode::Scooter,
        Some(everymap_core::domains::routing::TransportMode::Bus) => TransportMode::Bus,
        Some(everymap_core::domains::routing::TransportMode::Taxi) => TransportMode::Taxi,
        Some(everymap_core::domains::routing::TransportMode::Unknown) => TransportMode::Car,
    };

    let mut here_opts = HereRouteOptions {
        transport_mode,
        alternatives: opts.alternatives,
        departure_time: opts.departure_time.clone(),
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Convert avoid types
    if !opts.avoid.is_empty() {
        here_opts.avoid = Some(opts.avoid.iter().map(|a| match a {
            everymap_core::domains::routing::AvoidType::Tolls => "tolls".to_string(),
            everymap_core::domains::routing::AvoidType::Ferries => "ferries".to_string(),
            everymap_core::domains::routing::AvoidType::Tunnels => "tunnels".to_string(),
            everymap_core::domains::routing::AvoidType::Highways => "highways".to_string(),
            everymap_core::domains::routing::AvoidType::DirtRoads => "dirtRoads".to_string(),
        }).collect());
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
                here_opts.via = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("arrival_time").and_then(|v| v.as_str()) {
                here_opts.arrival_time = Some(v.to_string());
            }
            if let Some(v) = obj.get("exclude").and_then(|v| v.as_array()) {
                here_opts.exclude = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                here_opts.units = serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("spans").and_then(|v| v.as_array()) {
                here_opts.spans = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("vehicle").and_then(|v| v.as_array()) {
                here_opts.vehicle = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("consumption_model").and_then(|v| v.as_str()) {
                here_opts.consumption_model = serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("traffic").and_then(|v| v.as_str()) {
                here_opts.traffic = serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
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
                here_opts.max_speed_on_segment = Some(v.iter().filter_map(|i| serde_json::from_value(i.clone()).ok()).collect());
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
                let parsed: Vec<ReturnField> = v.iter()
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
    async fn calculate_route(&self, start: &everymap_core::types::Coordinate, end: &everymap_core::types::Coordinate, options: &RouteOptions) -> EveryMapResult<RouteResponse> {
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

        let mut params: Vec<(&str, String)> = vec![
            ("transportMode", transport.to_string()),
            ("routingMode", mode.to_string()),
            ("origin", format!("{},{}", start.lat, start.lng)),
            ("destination", format!("{},{}", end.lat, end.lng)),
        ];

        if let Some(ret) = &here_opts.return_fields {
            params.push(("return", ret.iter().map(|r| serde_json::to_value(r).unwrap().as_str().unwrap().to_string()).collect::<Vec<_>>().join(",")));
        } else {
            params.push(("return", "polyline,summary".to_string()));
        }

        if let Some(alts) = here_opts.alternatives {
            params.push(("alternatives", alts.to_string()));
        }
        if let Some(via) = &here_opts.via {
            for v in via {
                params.push(("via", v.clone()));
            }
        }
        if let Some(dt) = &here_opts.departure_time {
            params.push(("departureTime", dt.clone()));
        }
        if let Some(at) = &here_opts.arrival_time {
            params.push(("arrivalTime", at.clone()));
        }
        if let Some(avoid) = &here_opts.avoid {
            params.push(("avoid", avoid.join(",")));
        }
        if let Some(exclude) = &here_opts.exclude {
            params.push(("exclude", exclude.join(",")));
        }
        if let Some(units) = &here_opts.units {
            params.push(("units", serde_json::to_value(units).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(lang) = &here_opts.lang {
            params.push(("lang", lang.clone()));
        }
        if let Some(spans) = &here_opts.spans {
            params.push(("spans", spans.join(",")));
        }
        if let Some(vehicle) = &here_opts.vehicle {
            params.push(("vehicle", vehicle.join(",")));
        }
        if let Some(cm) = &here_opts.consumption_model {
            params.push(("consumptionModel", serde_json::to_value(cm).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(traffic) = &here_opts.traffic {
            params.push(("traffic", serde_json::to_value(traffic).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(bt) = &here_opts.billing_tag {
            params.push(("billingTag", bt.clone()));
        }
        if let Some(currency) = &here_opts.currency {
            params.push(("currency", currency.clone()));
        }
        if let Some(customizations) = &here_opts.customizations {
            params.push(("customizations", customizations.clone()));
        }
        if let Some(rh) = &here_opts.route_handle {
            params.push(("routeHandle", rh.clone()));
        }

        let url = format!("{}/routes", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereRouteApiResponse = response.json().await?;

        let routes: Vec<RouteResult> = here_res.routes.into_iter()
            .map(RouteResult::from)
            .collect();

        Ok(RouteResponse { routes })
    }
}