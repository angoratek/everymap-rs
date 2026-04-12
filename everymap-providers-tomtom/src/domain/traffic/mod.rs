pub mod types;

use async_trait::async_trait;
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse, TrafficFlow, TrafficIncident, IncidentSeverity};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::TomTomClient;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://api.tomtom.com";

/// Implementation of TrafficProvider for TomTom Traffic API.
pub struct TomTomTraffic {
    client: Arc<TomTomClient>,
    base_url: String,
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

/// Map TomTom severity string to core IncidentSeverity.
fn map_severity(severity: &str) -> IncidentSeverity {
    match severity.to_lowercase().as_str() {
        "minor" => IncidentSeverity::Minor,
        "moderate" => IncidentSeverity::Minor,
        "major" => IncidentSeverity::Major,
        "critical" => IncidentSeverity::Critical,
        _ => IncidentSeverity::Unknown,
    }
}

/// Calculate jam factor from current/free-flow speed ratio.
fn calculate_jam_factor(current: f64, free_flow: f64) -> f64 {
    if free_flow <= 0.0 { return 0.0; }
    let ratio = current / free_flow;
    if ratio >= 1.0 { 0.0 } else { (1.0 - ratio) * 10.0 }
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

        let flows: Vec<TrafficFlow> = flow_res.flow_segment_data.map(|seg| {
            let jam_factor = calculate_jam_factor(seg.current_speed, seg.free_flow_speed);
            TrafficFlow {
                speed: Some(seg.current_speed),
                free_flow_speed: Some(seg.free_flow_speed),
                jam_factor: Some(jam_factor),
                confidence: seg.confidence,
                road_name: seg.road_name,
            }
        }).into_iter().collect();

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

                inc_res.incidents.into_iter().map(|inc| {
                    TrafficIncident {
                        id: inc.id,
                        incident_type: inc.incident_type,
                        severity: inc.severity.as_deref().map(map_severity),
                        description: inc.description,
                        road_name: None,
                        start_time: inc.start_time,
                        end_time: inc.end_time,
                    }
                }).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(TrafficResponse { flows, incidents, raw: None })
    }
}