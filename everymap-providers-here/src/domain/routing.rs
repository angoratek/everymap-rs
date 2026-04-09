pub mod types;

use async_trait::async_trait;
use everymap_core::domains::routing::{Router, RouteRequest, RouteResponse};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Polyline;
use crate::client::HereClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use types::*;

const ROUTING_BASE_URL: &str = "https://router.hereapi.com/v8";

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

#[async_trait]
impl Router for HereRouter {
    type Options = HereRouteOptions;
    type Response = RouteResponse;

    async fn calculate_route(&self, req: RouteRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let transport = match req.options.transport_mode {
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

        let mode = match req.options.routing_mode {
            RoutingMode::Fast => "fast",
            RoutingMode::Short => "short",
        };

        let mut params: Vec<(&str, String)> = vec![
            ("transportMode", transport.to_string()),
            ("routingMode", mode.to_string()),
            ("origin", format!("{},{}", req.start.lat, req.start.lng)),
            ("destination", format!("{},{}", req.end.lat, req.end.lng)),
        ];

        if let Some(ret) = &req.options.return_fields {
            params.push(("return", ret.iter().map(|r| serde_json::to_value(r).unwrap().as_str().unwrap().to_string()).collect::<Vec<_>>().join(",")));
        } else {
            params.push(("return", "polyline,summary".to_string()));
        }

        if let Some(alts) = req.options.alternatives {
            params.push(("alternatives", alts.to_string()));
        }
        if let Some(via) = &req.options.via {
            for v in via {
                params.push(("via", v.clone()));
            }
        }
        if let Some(dt) = &req.options.departure_time {
            params.push(("departureTime", dt.clone()));
        }
        if let Some(at) = &req.options.arrival_time {
            params.push(("arrivalTime", at.clone()));
        }
        if let Some(avoid) = &req.options.avoid {
            params.push(("avoid", avoid.join(",")));
        }
        if let Some(exclude) = &req.options.exclude {
            params.push(("exclude", exclude.join(",")));
        }
        if let Some(units) = &req.options.units {
            params.push(("units", serde_json::to_value(units).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(lang) = &req.options.lang {
            params.push(("lang", lang.clone()));
        }
        if let Some(spans) = &req.options.spans {
            params.push(("spans", spans.join(",")));
        }
        if let Some(vehicle) = &req.options.vehicle {
            params.push(("vehicle", vehicle.join(",")));
        }
        if let Some(cm) = &req.options.consumption_model {
            params.push(("consumptionModel", serde_json::to_value(cm).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(traffic) = &req.options.traffic {
            params.push(("traffic", serde_json::to_value(traffic).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(bt) = &req.options.billing_tag {
            params.push(("billingTag", bt.clone()));
        }
        if let Some(currency) = &req.options.currency {
            params.push(("currency", currency.clone()));
        }
        if let Some(customizations) = &req.options.customizations {
            params.push(("customizations", customizations.clone()));
        }
        if let Some(rh) = &req.options.route_handle {
            params.push(("routeHandle", rh.clone()));
        }

        let url = format!("{}/routes", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereRouteApiResponse = response.json().await?;

        let route = here_res.routes.into_iter().next()
            .ok_or_else(|| everymap_core::error::EveryMapError::ProviderError("No routes found".to_string()))?;

        let section = route.sections.into_iter().next()
            .ok_or_else(|| everymap_core::error::EveryMapError::ProviderError("No route sections found".to_string()))?;

        let geometry = section.polyline
            .and_then(|p| p.polyline)
            .map(|encoded| {
                everymap_core::types::FlexiblePolyline::decode(&encoded)
                    .map(Polyline::new)
                    .unwrap_or_else(|_| Polyline::new(vec![]))
            })
            .unwrap_or_else(|| Polyline::new(vec![]));

        Ok(RouteResponse {
            distance: section.summary.length.unwrap_or(0.0),
            duration: section.summary.duration.unwrap_or(0.0),
            geometry,
        })
    }
}