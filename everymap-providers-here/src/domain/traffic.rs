pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::traffic::{
    TrafficFlow, TrafficOptions, TrafficProvider, TrafficResponse,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use std::sync::Arc;

pub use types::*;

const TRAFFIC_BASE_URL: &str = "https://data.traffic.hereapi.com/v7";

/// Metres in one degree of latitude at the WGS 84 mean radius. The contract
/// states a traffic radius in metres while HERE Traffic takes a rectangle in
/// degrees, so a radius becomes a latitude offset through this value.
const METRES_PER_DEGREE_LATITUDE: f64 = 111_320.0;

/// Builds the HERE `in` area filter for a traffic query.
///
/// A caller-supplied bounding box is used verbatim. Otherwise the contract's
/// radius, which is in metres, becomes a latitude/longitude offset around the
/// coordinate. HERE Traffic has no default area, so a query carrying neither a
/// box nor a radius is refused instead of being answered for guessed ground.
///
/// # Parameters
/// - `location` — the centre a radius is measured from.
/// - `options` — the caller's area bounds.
///
/// # Returns
/// The `bbox:west,south,east,north` filter both traffic endpoints expect.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] when `options` carries neither a
/// `bounding_box` nor a `radius`.
fn traffic_area_filter(location: &Coordinate, options: &TrafficOptions) -> EveryMapResult<String> {
    if let Some(bounding_box) = &options.bounding_box {
        return Ok(format!(
            "bbox:{},{},{},{}",
            bounding_box.south_west.lng,
            bounding_box.south_west.lat,
            bounding_box.north_east.lng,
            bounding_box.north_east.lat,
        ));
    }
    let radius_metres = options.radius.ok_or_else(|| {
        EveryMapError::ValidationError(
            "traffic requires a radius in metres or a bounding box".to_string(),
        )
    })?;
    let latitude_offset = radius_metres / METRES_PER_DEGREE_LATITUDE;
    let longitude_offset =
        radius_metres / (METRES_PER_DEGREE_LATITUDE * location.lat.to_radians().cos());
    Ok(format!(
        "bbox:{},{},{},{}",
        location.lng - longitude_offset,
        location.lat - latitude_offset,
        location.lng + longitude_offset,
        location.lat + latitude_offset,
    ))
}

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
            .build_request(everymap_core::http::Method::GET, &url)
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
            .build_request(everymap_core::http::Method::GET, &url)
            .query(&params);

        let here_res: HereIncidentsResponse = self.client.request_json(builder).await?;

        Ok(here_res)
    }
}

/// Convert core `TrafficOptions` to HERE-specific `HereFlowOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn flow_options_from_core(options: &TrafficOptions) -> HereFlowOptions {
    let mut here_options = HereFlowOptions {
        ..Default::default()
    };

    if options.language.is_some() {
        log::warn!(
            "HERE traffic flow API does not support language in the flow endpoint; \
             language will be ignored for flow data. Use incidents API for localized incident data."
        );
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("in_filter").and_then(|v| v.as_str()) {
                here_options.in_filter = Some(v.to_string());
            }
            if let Some(v) = obj.get("location_referencing").and_then(|v| v.as_array()) {
                here_options.location_referencing = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("min_jam_factor").and_then(|v| v.as_f64()) {
                here_options.min_jam_factor = Some(v);
            }
            if let Some(v) = obj.get("max_jam_factor").and_then(|v| v.as_f64()) {
                here_options.max_jam_factor = Some(v);
            }
            if let Some(v) = obj.get("functional_classes").and_then(|v| v.as_array()) {
                here_options.functional_classes = Some(
                    v.iter()
                        .filter_map(|i| i.as_u64().map(|n| n as u32))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("advanced_features").and_then(|v| v.as_array()) {
                here_options.advanced_features = Some(
                    v.iter()
                        .filter_map(|i| serde_json::from_value(i.clone()).ok())
                        .collect(),
                );
            }
            if let Some(v) = obj.get("use_ref_replacements").and_then(|v| v.as_bool()) {
                here_options.use_ref_replacements = Some(v);
            }
            if let Some(v) = obj
                .get("exact_segment_ref_matching")
                .and_then(|v| v.as_bool())
            {
                here_options.exact_segment_ref_matching = Some(v);
            }
        }
    }

    here_options
}

#[async_trait]
impl TrafficProvider for HereTraffic {
    async fn get_traffic(
        &self,
        location: &Coordinate,
        options: &TrafficOptions,
    ) -> EveryMapResult<TrafficResponse> {
        // Convert core options to HERE-specific options and query exactly the
        // ground the caller asked about.
        let area_filter = traffic_area_filter(location, options)?;
        let mut here_options = flow_options_from_core(options);
        here_options.in_filter = Some(area_filter.clone());

        // Use the rich flow API and extract simplified data
        let flow_response = self.get_flow(*location, &here_options).await?;

        let flows: Vec<TrafficFlow> = flow_response.results.into_iter().map(Into::into).collect();

        // Fetch incidents when requested, over the same area as the flow query.
        let mut incidents = vec![];
        if options.include_incidents.unwrap_or(false) {
            let incidents_options = HereIncidentsOptions {
                in_filter: Some(area_filter),
                location_referencing: Some(vec![LocationReferencing::None]),
                ..Default::default()
            };
            let incident_response = self.get_incidents(&incidents_options).await?;
            incidents = incident_response
                .results
                .into_iter()
                .map(|item| item.incident.into())
                .collect();
        }

        Ok(TrafficResponse {
            flows,
            incidents,
            raw: None,
        })
    }
}
