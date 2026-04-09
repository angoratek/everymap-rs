pub mod types;

use async_trait::async_trait;
use everymap_core::domains::isoline::{IsolineProvider, IsolineRequest, IsolineResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
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
#[serde(rename_all = "snake_case")]
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
    client: Arc<HereClient>,
    base_url: String,
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

// Internal deserialization for backward-compatible core trait
#[derive(Debug, Deserialize)]
struct HereIsolineLegacyResponse {
    isolines: Vec<HereIsolineLegacy>,
}

#[derive(Debug, Deserialize)]
struct HereIsolineLegacy {
    #[serde(default)]
    polyline: Option<HereIsolinePolyline>,
}

#[derive(Debug, Deserialize)]
struct HereIsolinePolyline {
    #[serde(default)]
    outer: Option<String>,
}

#[async_trait]
impl IsolineProvider for HereIsoline {
    type Options = HereIsolineOptions;
    type Response = IsolineResponse;

    async fn get_isoline(&self, req: IsolineRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let range_type = match req.options.range_type {
            RangeType::Time => "time",
            RangeType::Distance => "distance",
            RangeType::Consumption => "consumption",
        };

        let transport = match req.options.transport_mode {
            IsolineTransportMode::Car => "car",
            IsolineTransportMode::Truck => "truck",
            IsolineTransportMode::Pedestrian => "pedestrian",
            IsolineTransportMode::Bicycle => "bicycle",
            IsolineTransportMode::Bus => "bus",
            IsolineTransportMode::PrivateBus => "privateBus",
            IsolineTransportMode::Scooter => "scooter",
            IsolineTransportMode::Taxi => "taxi",
        };

        let routing_mode = match req.options.routing_mode {
            IsolineRoutingMode::Fast => "fast",
            IsolineRoutingMode::Short => "short",
        };

        let mut params: Vec<(&str, String)> = vec![
            ("location", format!("{},{}", req.center.lat, req.center.lng)),
            ("range[type]", range_type.to_string()),
            ("range[values]", req.range.to_string()),
            ("transportMode", transport.to_string()),
            ("routingMode", routing_mode.to_string()),
            ("return", "polyline".to_string()),
        ];

        if let Some(opt) = &req.options.optimize_for {
            let val = serde_json::to_value(opt).unwrap().as_str().unwrap().to_string();
            params.push(("optimizeFor", val));
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
        if let Some(traffic) = &req.options.traffic {
            params.push(("traffic", serde_json::to_value(traffic).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(bt) = &req.options.billing_tag {
            params.push(("billingTag", bt.clone()));
        }

        let url = format!("{}/isolines", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereIsolineLegacyResponse = response.json().await?;

        let mut all_points = Vec::new();
        for iso in &here_res.isolines {
            if let Some(poly) = &iso.polyline {
                if let Some(encoded) = &poly.outer {
                    let points = everymap_core::types::FlexiblePolyline::decode(encoded)
                        .map_err(everymap_core::error::EveryMapError::ProviderError)?;
                    all_points.extend(points);
                }
            }
        }

        if all_points.is_empty() && !here_res.isolines.is_empty() {
            all_points.push(req.center);
        }

        Ok(IsolineResponse {
            polygon: all_points,
        })
    }
}