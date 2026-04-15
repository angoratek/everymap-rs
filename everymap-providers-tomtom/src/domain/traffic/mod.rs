pub mod types;

use async_trait::async_trait;
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse, TrafficFlow, TrafficIncident};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::TomTomClient;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of TrafficProvider for TomTom Traffic API.
pub struct TomTomTraffic {
    pub(crate) client: Arc<TomTomClient>,
    pub(crate) base_url: String,
}

impl TomTomTraffic {
    pub fn new(client: Arc<TomTomClient>) -> Self {
        Self {
            client,
            base_url: TRAFFIC_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<TomTomClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl TrafficProvider for TomTomTraffic {
    async fn get_traffic(&self, location: &Coordinate, options: &TrafficOptions) -> EveryMapResult<TrafficResponse> {
        // Get flow data
        let flow_url = format!("{}/traffic/services/4/flowSegmentData/absolute/10/json", self.base_url);
        let mut flow_params: Vec<(&str, String)> = vec![
            ("point", format!("{},{}", location.lat, location.lng)),
        ];
        if let Some(lang) = &options.language {
            flow_params.push(("language", lang.clone()));
        }

        let flow_builder = self.client.build_request(reqwest::Method::GET, &flow_url)
            .query(&flow_params);
        let flow_res: TomTomFlowResponse = self.client.request_json(flow_builder).await?;

        let flows: Vec<TrafficFlow> = flow_res.flow_segment_data
            .map(TrafficFlow::from)
            .into_iter()
            .collect();

        // Get incidents if requested
        let incidents: Vec<TrafficIncident> = if options.include_incidents.unwrap_or(false) {
            if let Some(radius) = options.radius {
                let inc_url = format!("{}/traffic/services/5/incidentDetails", self.base_url);
                let bbox = format!("{},{},{},{}",
                    location.lat - radius / 111000.0, location.lng - radius / 111000.0,
                    location.lat + radius / 111000.0, location.lng + radius / 111000.0);
                let inc_params: Vec<(&str, String)> = vec![
                    ("bbox", bbox),
                    ("fields", "{incidents{type,geometry{id,coordinates},severity,description,from,to,startTime,endTime}}".to_string()),
                    ("language", options.language.clone().unwrap_or_else(|| "en-US".to_string())),
                ];

                let inc_builder = self.client.build_request(reqwest::Method::GET, &inc_url)
                    .query(&inc_params);
                let inc_res: TomTomIncidentsResponse = self.client.request_json(inc_builder).await?;

                inc_res.incidents.into_iter()
                    .map(TrafficIncident::from)
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(TrafficResponse { flows, incidents, raw: None })
    }
}