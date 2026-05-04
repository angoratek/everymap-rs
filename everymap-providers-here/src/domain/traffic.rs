pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::traffic::{
    TrafficFlow, TrafficOptions, TrafficProvider, TrafficResponse,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://data.traffic.hereapi.com/v7";

/// Implementation of TrafficProvider for HERE Technologies.
///
/// In addition to the core `TrafficProvider` trait method (`get_traffic`),
/// this struct provides HERE-specific methods: `get_flow` and `get_incidents`.
pub struct HereTraffic {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
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
            params.push((
                "in",
                format!(
                    "bbox:{},{},{},{}",
                    location.lng - 0.01,
                    location.lat - 0.01,
                    location.lng + 0.01,
                    location.lat + 0.01
                ),
            ));
        }

        if let Some(location_referencing) = &options.location_referencing {
            let value = location_referencing
                .iter()
                .map(crate::util::enum_as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("locationReferencing", value));
        } else {
            params.push(("locationReferencing", "shape".to_string()));
        }

        if let Some(min_jf) = options.min_jam_factor {
            params.push(("minJamFactor", min_jf.to_string()));
        }
        if let Some(max_jf) = options.max_jam_factor {
            params.push(("maxJamFactor", max_jf.to_string()));
        }
        if let Some(functional_classes) = &options.functional_classes {
            let value = functional_classes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("functionalClasses", value));
        }
        if let Some(advanced_features) = &options.advanced_features {
            let value = advanced_features
                .iter()
                .map(crate::util::enum_as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("advancedFeatures", value));
        }
        if let Some(use_ref_replacements) = options.use_ref_replacements {
            params.push(("useRefReplacements", use_ref_replacements.to_string()));
        }
        if let Some(exact_segment_ref_matching) = options.exact_segment_ref_matching {
            params.push((
                "exactSegmentRefMatching",
                exact_segment_ref_matching.to_string(),
            ));
        }

        let url = format!("{}/flow", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereFlowResponse = self.client.request_json(builder).await?;

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

        if let Some(location_referencing) = &options.location_referencing {
            let value = location_referencing
                .iter()
                .map(crate::util::enum_as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("locationReferencing", value));
        } else {
            params.push(("locationReferencing", "shape".to_string()));
        }

        if let Some(functional_classes) = &options.functional_classes {
            let value = functional_classes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push(("functionalClasses", value));
        }
        if let Some(criticality) = &options.criticality {
            let value = criticality
                .iter()
                .map(crate::util::enum_as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("criticality", value));
        }
        if let Some(incident_types) = &options.incident_types {
            let value = incident_types
                .iter()
                .map(crate::util::enum_as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("type", value));
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
            params.push(("units", crate::util::enum_as_str(units)));
        }
        if let Some(use_ref_replacements) = options.use_ref_replacements {
            params.push(("useRefReplacements", use_ref_replacements.to_string()));
        }
        if let Some(exact_segment_ref_matching) = options.exact_segment_ref_matching {
            params.push((
                "exactSegmentRefMatching",
                exact_segment_ref_matching.to_string(),
            ));
        }

        let url = format!("{}/incidents", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let here_res: HereIncidentsResponse = self.client.request_json(builder).await?;

        Ok(here_res)
    }
}

/// Convert core `TrafficOptions` to HERE-specific `HereFlowOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn flow_options_from_core(options: &TrafficOptions) -> HereFlowOptions {
    let mut here_opts = HereFlowOptions {
        ..Default::default()
    };

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("in_filter").and_then(|v| v.as_str()) {
                here_opts.in_filter = Some(v.to_string());
            }
            if let Some(v) = obj.get("location_referencing").and_then(|v| v.as_array()) {
                here_opts.location_referencing = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("min_jam_factor").and_then(|v| v.as_f64()) {
                here_opts.min_jam_factor = Some(v);
            }
            if let Some(v) = obj.get("max_jam_factor").and_then(|v| v.as_f64()) {
                here_opts.max_jam_factor = Some(v);
            }
            if let Some(v) = obj.get("functional_classes").and_then(|v| v.as_array()) {
                here_opts.functional_classes = Some(
                    v.iter()
                        .filter_map(|i| i.as_u64().map(|n| n as u32))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("advanced_features").and_then(|v| v.as_array()) {
                here_opts.advanced_features = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("use_ref_replacements").and_then(|v| v.as_bool()) {
                here_opts.use_ref_replacements = Some(v);
            }
            if let Some(v) = obj
                .get("exact_segment_ref_matching")
                .and_then(|v| v.as_bool())
            {
                here_opts.exact_segment_ref_matching = Some(v);
            }
        }
    }

    here_opts
}

#[async_trait]
impl TrafficProvider for HereTraffic {
    async fn get_traffic(
        &self,
        location: &Coordinate,
        options: &TrafficOptions,
    ) -> EveryMapResult<TrafficResponse> {
        // Convert core options to HERE-specific options
        let here_opts = flow_options_from_core(options);

        // Use the rich flow API and extract simplified data
        let flow_response = self.get_flow(*location, &here_opts).await?;

        let flows: Vec<TrafficFlow> = flow_response.results.into_iter().map(Into::into).collect();

        // Fetch incidents when requested
        let mut incidents = vec![];
        if options.include_incidents.unwrap_or(false) {
            // Build a bounding box around the location using the radius or a default
            let radius_km = options.radius.unwrap_or(10.0);
            let lat_offset = radius_km / 111.32;
            let lng_offset = radius_km / (111.32 * location.lat.to_radians().cos());
            let bbox = format!(
                "bbox:{},{},{},{}",
                location.lng - lng_offset,
                location.lat - lat_offset,
                location.lng + lng_offset,
                location.lat + lat_offset,
            );
            let incidents_options = HereIncidentsOptions {
                in_filter: Some(bbox),
                location_referencing: Some(vec![LocationReferencing::None]),
                ..Default::default()
            };
            if let Ok(incident_response) = self.get_incidents(&incidents_options).await {
                incidents = incident_response.results.into_iter().map(|item| item.incident.into()).collect();
            }
        }

        Ok(TrafficResponse {
            flows,
            incidents,
            raw: None,
        })
    }
}
