pub mod types;

use async_trait::async_trait;
use everymap_core::domains::traffic::{TrafficProvider, TrafficRequest, TrafficResponse};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use crate::client::HereClient;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://data.traffic.hereapi.com/v7";

/// Implementation of TrafficProvider for HERE Technologies.
///
/// In addition to the core `TrafficProvider` trait method (`get_traffic`),
/// this struct provides HERE-specific methods: `get_flow` and `get_incidents`.
pub struct HereTraffic {
    client: Arc<HereClient>,
    base_url: String,
}

impl HereTraffic {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: TRAFFIC_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Get traffic flow data for an area.
    pub async fn get_flow(
        &self,
        location: Coordinate,
        options: &HereFlowOptions,
    ) -> EveryMapResult<HereFlowResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(in_filter) = &options.in_filter {
            params.push(("in", in_filter.clone()));
        } else {
            params.push(("in", format!("bbox:{},{},{},{}",
                location.lng - 0.01, location.lat - 0.01,
                location.lng + 0.01, location.lat + 0.01)));
        }

        if let Some(lr) = &options.location_referencing {
            let val = lr.iter()
                .map(|l| serde_json::to_value(l).unwrap().as_str().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("locationReferencing", val));
        } else {
            params.push(("locationReferencing", "shape".to_string()));
        }

        if let Some(min_jf) = options.min_jam_factor {
            params.push(("minJamFactor", min_jf.to_string()));
        }
        if let Some(max_jf) = options.max_jam_factor {
            params.push(("maxJamFactor", max_jf.to_string()));
        }
        if let Some(fc) = &options.functional_classes {
            let val = fc.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
            params.push(("functionalClasses", val));
        }
        if let Some(af) = &options.advanced_features {
            let val = af.iter()
                .map(|a| serde_json::to_value(a).unwrap().as_str().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("advancedFeatures", val));
        }
        if let Some(urr) = options.use_ref_replacements {
            params.push(("useRefReplacements", urr.to_string()));
        }
        if let Some(esrm) = options.exact_segment_ref_matching {
            params.push(("exactSegmentRefMatching", esrm.to_string()));
        }

        let url = format!("{}/flow", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereFlowResponse = response.json().await?;

        Ok(here_res)
    }

    /// Get traffic incidents for an area.
    pub async fn get_incidents(
        &self,
        options: &HereIncidentsOptions,
    ) -> EveryMapResult<HereIncidentsResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(in_filter) = &options.in_filter {
            params.push(("in", in_filter.clone()));
        }

        if let Some(lr) = &options.location_referencing {
            let val = lr.iter()
                .map(|l| serde_json::to_value(l).unwrap().as_str().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("locationReferencing", val));
        } else {
            params.push(("locationReferencing", "shape".to_string()));
        }

        if let Some(fc) = &options.functional_classes {
            let val = fc.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
            params.push(("functionalClasses", val));
        }
        if let Some(crit) = &options.criticality {
            let val = crit.iter()
                .map(|c| serde_json::to_value(c).unwrap().as_str().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("criticality", val));
        }
        if let Some(it) = &options.incident_types {
            let val = it.iter()
                .map(|t| serde_json::to_value(t).unwrap().as_str().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("type", val));
        }
        if let Some(est) = &options.earliest_start_time {
            params.push(("earliestStartTime", est.clone()));
        }
        if let Some(let_) = &options.latest_end_time {
            params.push(("latestEndTime", let_.clone()));
        }
        if let Some(lang) = &options.lang {
            params.push(("lang", lang.clone()));
        }
        if let Some(units) = &options.units {
            params.push(("units", serde_json::to_value(units).unwrap().as_str().unwrap().to_string()));
        }
        if let Some(urr) = options.use_ref_replacements {
            params.push(("useRefReplacements", urr.to_string()));
        }
        if let Some(esrm) = options.exact_segment_ref_matching {
            params.push(("exactSegmentRefMatching", esrm.to_string()));
        }

        let url = format!("{}/incidents", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let here_res: HereIncidentsResponse = response.json().await?;

        Ok(here_res)
    }
}

#[async_trait]
impl TrafficProvider for HereTraffic {
    type Options = HereFlowOptions;
    type Response = TrafficResponse;

    async fn get_traffic(&self, req: TrafficRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        // Use the rich flow API and extract simplified data
        let flow_res = self.get_flow(req.location, &req.options).await?;

        let jam_factor = flow_res.results.first()
            .and_then(|item| item.current_flow.jam_factor)
            .unwrap_or(0.0);

        Ok(TrafficResponse {
            jam_factor,
            incidents: vec![],
        })
    }
}