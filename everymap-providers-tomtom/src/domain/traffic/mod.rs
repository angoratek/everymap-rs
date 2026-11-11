pub mod types;

use crate::client::TomTomClient;
use async_trait::async_trait;
use everymap_core::domains::traffic::{
    TrafficFlow, TrafficIncident, TrafficOptions, TrafficProvider, TrafficResponse,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://api.tomtom.com";

/// Metres per degree of latitude on a spherical Earth, used to turn a search
/// radius into the incident endpoint's bounding box.
const METRES_PER_DEGREE_LATITUDE: f64 = 111_000.0;

/// Floor applied to the cosine of the latitude when scaling a radius into
/// longitude degrees, so a query near a pole cannot divide by zero.
const MINIMUM_LONGITUDE_SCALE: f64 = 0.01;

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
    async fn get_traffic(
        &self,
        location: &Coordinate,
        options: &TrafficOptions,
    ) -> EveryMapResult<TrafficResponse> {
        // Get flow data
        let flow_url = format!(
            "{}/traffic/services/4/flowSegmentData/absolute/10/json",
            self.base_url
        );
        let mut flow_params: Vec<(&str, String)> =
            vec![("point", format!("{},{}", location.lat, location.lng))];
        if let Some(lang) = &options.language {
            flow_params.push(("language", lang.clone()));
        }
        if options.provider_extra.is_some() {
            log::warn!("TomTom Traffic API does not support provider_extra; ignoring");
        }

        let flow_builder = self
            .client
            .build_request(everymap_core::http::Method::GET, &flow_url)
            .query(&flow_params);
        let flow_res: TomTomFlowResponse = self.client.request_parsed_json(flow_builder).await?;

        let flows: Vec<TrafficFlow> = flow_res
            .flow_segment_data
            .map(TrafficFlow::from)
            .into_iter()
            .collect();

        // Get incidents if requested
        let incidents: Vec<TrafficIncident> = if options.include_incidents.unwrap_or(false) {
            // TomTom's incident bbox is `minLon,minLat,maxLon,maxLat`. A
            // caller-supplied box wins over the radius, so a map viewport asks
            // for exactly the ground it draws.
            let incident_bounding_box = if let Some(bounding_box) = &options.bounding_box {
                Some(format!(
                    "{},{},{},{}",
                    bounding_box.south_west.lng,
                    bounding_box.south_west.lat,
                    bounding_box.north_east.lng,
                    bounding_box.north_east.lat
                ))
            } else {
                options.radius.map(|radius| {
                    let lat_offset = radius / METRES_PER_DEGREE_LATITUDE;
                    let lng_offset = radius
                        / (METRES_PER_DEGREE_LATITUDE
                            * location.lat.to_radians().cos().max(MINIMUM_LONGITUDE_SCALE));
                    format!(
                        "{},{},{},{}",
                        location.lng - lng_offset,
                        location.lat - lat_offset,
                        location.lng + lng_offset,
                        location.lat + lat_offset
                    )
                })
            };
            if let Some(bbox) = incident_bounding_box {
                let inc_url = format!("{}/traffic/services/5/incidentDetails", self.base_url);
                let inc_params: Vec<(&str, String)> = vec![
                    ("bbox", bbox),
                    ("fields", "{incidents{type,geometry{id,coordinates},severity,description,from,to,startTime,endTime}}".to_string()),
                    ("language", options.language.clone().unwrap_or_else(|| "en-US".to_string())),
                ];

                let inc_builder = self
                    .client
                    .build_request(everymap_core::http::Method::GET, &inc_url)
                    .query(&inc_params);
                let inc_res: TomTomIncidentsResponse =
                    self.client.request_parsed_json(inc_builder).await?;

                inc_res
                    .incidents
                    .into_iter()
                    .map(TrafficIncident::from)
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(TrafficResponse {
            flows,
            incidents,
            raw: None,
        })
    }
}
