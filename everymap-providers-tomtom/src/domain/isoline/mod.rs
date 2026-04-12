pub mod types;

use async_trait::async_trait;
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, IsolineResponse, IsolineResult, RangeType};
use everymap_core::domains::routing::TransportMode;
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::TomTomClient;
use std::sync::Arc;

pub use types::*;

const ROUTING_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of IsolineProvider for TomTom Reachable Range API.
pub struct TomTomIsoline {
    client: Arc<TomTomClient>,
    base_url: String,
}

impl TomTomIsoline {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: ROUTING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl IsolineProvider for TomTomIsoline {
    async fn get_isoline(&self, center: &Coordinate, range: f64, options: &IsolineOptions) -> EveryMapResult<IsolineResponse> {
        let url = format!("{}/routing/1/calculateReachableRange/{},{}/json", self.base_url, center.lat, center.lng);

        let range_type_str = match options.range_type.as_ref() {
            Some(RangeType::Time) => "time",
            Some(RangeType::Consumption) => "energy",
            _ => "distance",
        };

        let mut params: Vec<(&str, String)> = vec![
            (range_type_str, range.to_string()),
        ];

        if let Some(mode) = &options.transport_mode {
            let mode_str = match mode {
                TransportMode::Car => "car",
                TransportMode::Truck => "truck",
                TransportMode::Pedestrian => "pedestrian",
                TransportMode::Bicycle => "bicycle",
                _ => "car",
            };
            params.push(("travelMode", mode_str.to_string()));
        }

        if let Some(departure) = &options.departure_time {
            params.push(("departAt", departure.clone()));
        }

        if let Some(extra) = &options.provider_extra {
            if let Some(obj) = extra.as_object() {
                if let Some(v) = obj.get("routeType").and_then(|v| v.as_str()) {
                    params.push(("routeType", v.to_string()));
                }
                if let Some(v) = obj.get("vehicleMaxSpeed").and_then(|v| v.as_u64()) {
                    params.push(("vehicleMaxSpeed", v.to_string()));
                }
            }
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: TomTomReachableRangeResponse = self.client.request_json(builder).await?;

        let isolines: Vec<IsolineResult> = result.reachable_range.map(|rr| {
            let polygon: Vec<Coordinate> = rr.boundary.into_iter()
                .map(|b| Coordinate::new(b.lat, b.lng).unwrap_or_else(|_| Coordinate::new(0.0, 0.0).unwrap()))
                .collect();
            vec![IsolineResult {
                range: Some(range),
                polygon,
            }]
        }).unwrap_or_default();

        Ok(IsolineResponse { isolines, raw: None })
    }
}