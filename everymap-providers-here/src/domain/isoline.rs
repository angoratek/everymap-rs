pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::isoline::{
    IsolineOptions, IsolineProvider, IsolineResponse, IsolineResult,
};
use everymap_core::error::EveryMapResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use types::*;

const ISOLINE_BASE_URL: &str = "https://isoline.router.hereapi.com/v8";

/// Exhaustive options for HERE Isoline Routing API v8.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HereIsolineOptions {
    pub range_type: RangeType,
    pub transport_mode: IsolineTransportMode,
    pub routing_mode: IsolineRoutingMode,
    pub optimize_for: Option<OptimizeFor>,
    pub departure_time: Option<String>,
    pub arrival_time: Option<String>,
    pub avoid: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub shape: Option<String>,
    pub vehicle: Option<Vec<String>>,
    pub consumption_model: Option<ConsumptionModel>,
    pub ev: Option<Vec<String>>,
    pub fuel: Option<Vec<String>>,
    pub max_speed_on_segment: Option<Vec<String>>,
    pub taxi: Option<Vec<String>>,
    pub traffic: Option<TrafficMode>,
    pub billing_tag: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RangeType {
    #[default]
    Time,
    Distance,
    Consumption,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IsolineTransportMode {
    #[default]
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Bus,
    PrivateBus,
    Scooter,
    Taxi,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolineRoutingMode {
    #[default]
    Fast,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizeFor {
    Quality,
    Performance,
    Balanced,
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

/// Implementation of IsolineProvider for HERE Technologies.
pub struct HereIsoline {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereIsoline {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: ISOLINE_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

// Internal deserialization for the real v8 API response
// The HERE Isoline API v8 returns:
// {"isolines": [{"range": {"type": "...", "value": N}, "polygons": [{"outer": "flexible_polyline"}]}]}
#[derive(Debug, Deserialize)]
struct HereIsolineLegacyResponse {
    #[serde(default)]
    isolines: Vec<HereIsolineLegacy>,
}

#[derive(Debug, Deserialize)]
struct HereIsolineLegacy {
    #[serde(default)]
    range: Option<HereApiIsolineRange>,
    #[serde(default)]
    polygons: Vec<HereIsolinePolygon>,
    // Also support the old "polyline" field for backward compatibility
    #[serde(default)]
    polyline: Option<HereIsolinePolyline>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HereApiIsolineRange {
    #[serde(default)]
    #[serde(rename = "type")]
    type_: Option<String>,
    #[serde(default)]
    value: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct HereIsolinePolygon {
    #[serde(default)]
    outer: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HereIsolinePolyline {
    #[serde(default)]
    outer: Option<String>,
}

impl From<HereIsolineLegacy> for IsolineResult {
    fn from(iso: HereIsolineLegacy) -> Self {
        // Try "polygons[].outer" first (v8 format), fall back to "polyline.outer"
        let encoded = iso
            .polygons
            .iter()
            .filter_map(|p| p.outer.as_ref())
            .next()
            .or_else(|| iso.polyline.as_ref().and_then(|p| p.outer.as_ref()));

        let polygon = encoded
            .map(|enc| {
                everymap_core::types::FlexiblePolyline::decode(enc).unwrap_or_else(|_| vec![])
            })
            .unwrap_or_default();

        let range = iso.range.and_then(|r| r.value);

        Self { range, polygon }
    }
}

/// Convert core `IsolineOptions` to HERE-specific `HereIsolineOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn isoline_options_from_core(opts: &IsolineOptions) -> HereIsolineOptions {
    let range_type = match opts.range_type {
        Some(everymap_core::domains::isoline::RangeType::Distance) | None => RangeType::Distance,
        Some(everymap_core::domains::isoline::RangeType::Time) => RangeType::Time,
        Some(everymap_core::domains::isoline::RangeType::Consumption) => RangeType::Consumption,
    };

    let transport_mode = match opts.transport_mode {
        Some(everymap_core::domains::routing::TransportMode::Car) | None => {
            IsolineTransportMode::Car
        }
        Some(everymap_core::domains::routing::TransportMode::Truck) => IsolineTransportMode::Truck,
        Some(everymap_core::domains::routing::TransportMode::Pedestrian) => {
            IsolineTransportMode::Pedestrian
        }
        Some(everymap_core::domains::routing::TransportMode::Bicycle) => {
            IsolineTransportMode::Bicycle
        }
        Some(everymap_core::domains::routing::TransportMode::Bus) => IsolineTransportMode::Bus,
        Some(everymap_core::domains::routing::TransportMode::Scooter) => {
            IsolineTransportMode::Scooter
        }
        Some(everymap_core::domains::routing::TransportMode::Taxi) => IsolineTransportMode::Taxi,
        Some(everymap_core::domains::routing::TransportMode::Unknown) => IsolineTransportMode::Car,
    };

    let mut here_opts = HereIsolineOptions {
        range_type,
        transport_mode,
        departure_time: opts.departure_time.clone(),
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
                    "short" => IsolineRoutingMode::Short,
                    _ => IsolineRoutingMode::Fast,
                };
            }
            if let Some(v) = obj.get("optimize_for") {
                here_opts.optimize_for = serde_json::from_value(v.clone()).ok();
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
            if let Some(v) = obj.get("shape").and_then(|v| v.as_str()) {
                here_opts.shape = Some(v.to_string());
            }
            if let Some(v) = obj.get("vehicle").and_then(|v| v.as_array()) {
                here_opts.vehicle = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("consumption_model") {
                here_opts.consumption_model = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("ev").and_then(|v| v.as_array()) {
                here_opts.ev = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("fuel").and_then(|v| v.as_array()) {
                here_opts.fuel = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("max_speed_on_segment").and_then(|v| v.as_array()) {
                here_opts.max_speed_on_segment = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("taxi").and_then(|v| v.as_array()) {
                here_opts.taxi = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("traffic").and_then(|v| v.as_str()) {
                here_opts.traffic =
                    serde_json::from_value(serde_json::Value::String(v.to_string())).ok();
            }
            if let Some(v) = obj.get("billing_tag").and_then(|v| v.as_str()) {
                here_opts.billing_tag = Some(v.to_string());
            }
        }
    }

    here_opts
}

#[async_trait]
impl IsolineProvider for HereIsoline {
    async fn get_isoline(
        &self,
        center: &everymap_core::types::Coordinate,
        range: f64,
        options: &IsolineOptions,
    ) -> EveryMapResult<IsolineResponse> {
        let here_opts = isoline_options_from_core(options);

        let range_type = match here_opts.range_type {
            RangeType::Time => "time",
            RangeType::Distance => "distance",
            RangeType::Consumption => "consumption",
        };

        let transport = match here_opts.transport_mode {
            IsolineTransportMode::Car => "car",
            IsolineTransportMode::Truck => "truck",
            IsolineTransportMode::Pedestrian => "pedestrian",
            IsolineTransportMode::Bicycle => "bicycle",
            IsolineTransportMode::Bus => "bus",
            IsolineTransportMode::PrivateBus => "privateBus",
            IsolineTransportMode::Scooter => "scooter",
            IsolineTransportMode::Taxi => "taxi",
        };

        let routing_mode = match here_opts.routing_mode {
            IsolineRoutingMode::Fast => "fast",
            IsolineRoutingMode::Short => "short",
        };

        let mut params: Vec<(&str, String)> = vec![
            ("origin", format!("{},{}", center.lat, center.lng)),
            ("range[type]", range_type.to_string()),
            ("range[values]", range.to_string()),
            ("transportMode", transport.to_string()),
            ("routingMode", routing_mode.to_string()),
            ("return", "polyline".to_string()),
        ];

        if let Some(opt) = &here_opts.optimize_for {
            let value = crate::util::enum_as_str(opt);
            params.push(("optimizeFor", value));
        }
        if let Some(departure_time) = &here_opts.departure_time {
            params.push(("departureTime", departure_time.clone()));
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
        if let Some(traffic) = &here_opts.traffic {
            params.push(("traffic", crate::util::enum_as_str(traffic)));
        }
        if let Some(shape) = &here_opts.shape {
            params.push(("shape", shape.clone()));
        }
        if let Some(vehicle) = &here_opts.vehicle {
            params.push(("vehicle", vehicle.join(",")));
        }
        if let Some(consumption_model) = &here_opts.consumption_model {
            params.push(("consumptionModel", crate::util::enum_as_str(consumption_model)));
        }
        if let Some(ev) = &here_opts.ev {
            params.push(("ev", ev.join(",")));
        }
        if let Some(fuel) = &here_opts.fuel {
            params.push(("fuel", fuel.join(",")));
        }
        if let Some(max_speed_on_segment) = &here_opts.max_speed_on_segment {
            params.push(("maxSpeedOnSegment", max_speed_on_segment.join(",")));
        }
        if let Some(taxi) = &here_opts.taxi {
            params.push(("taxi", taxi.join(",")));
        }
        if let Some(billing_tag) = &here_opts.billing_tag {
            params.push(("billingTag", billing_tag.clone()));
        }

        let url = format!("{}/isolines", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereIsolineLegacyResponse = self.client.request_json(builder).await?;

        let isolines: Vec<IsolineResult> = here_res
            .isolines
            .into_iter()
            .map(IsolineResult::from)
            .collect();

        Ok(IsolineResponse {
            isolines,
            raw: None,
        })
    }
}
