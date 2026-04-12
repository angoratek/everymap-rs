pub mod types;

use async_trait::async_trait;
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
use std::sync::Arc;

pub use types::*;

const ATTRIBUTES_BASE_URL: &str = "https://smap.hereapi.com/v8";

/// Options for HERE Map Attributes API v8.
#[derive(Debug, Clone, Default)]
pub struct HereAttributeOptions {
    /// The attribute layer to query.
    pub layer: AttributeLayer,
    /// Response format.
    pub format: AttributeFormat,
    /// Bounding box as "south,west;north,east" or "lat,lng;lat,lng".
    pub bbox: Option<String>,
    /// Specific feature IDs to retrieve.
    pub ids: Option<Vec<String>>,
    /// Spatial reference system (e.g., "EPSG:4326").
    pub srs: Option<String>,
    /// Language for localized strings.
    pub lang: Option<String>,
    /// Political view for disputed borders.
    pub political_view: Option<String>,
    /// Include additional attribute fields.
    pub include: Option<Vec<String>>,
    /// Exclude attribute fields.
    pub exclude: Option<Vec<String>>,
}

/// Implementation of AttributeProvider for HERE Technologies.
pub struct HereAttributeProvider {
    client: Arc<HereClient>,
    base_url: String,
}

impl HereAttributeProvider {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: ATTRIBUTES_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Get road attributes for a bounding box.
    /// Convenience method that returns typed road attribute data.
    pub async fn get_road_attributes(&self, bbox: &str, includes: Option<Vec<String>>) -> EveryMapResult<HereRoadAttributesResponse> {
        let url = format!("{}/attributes/roads", self.base_url);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), "json".to_string())];
        params.push(("bbox".to_string(), bbox.to_string()));
        if let Some(inc) = includes {
            params.push(("include".to_string(), inc.join(",")));
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get segment (topology) attributes for a bounding box.
    pub async fn get_segment_attributes(&self, bbox: &str, includes: Option<Vec<String>>) -> EveryMapResult<HereSegmentAttributesResponse> {
        let url = format!("{}/attributes/segments", self.base_url);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), "json".to_string())];
        params.push(("bbox".to_string(), bbox.to_string()));
        if let Some(inc) = includes {
            params.push(("include".to_string(), inc.join(",")));
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get administrative area attributes for a bounding box.
    pub async fn get_admin_areas(&self, bbox: &str) -> EveryMapResult<HereAdminAreasResponse> {
        let url = format!("{}/attributes/adminAreas", self.base_url);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), "json".to_string())];
        params.push(("bbox".to_string(), bbox.to_string()));

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get building attributes for a bounding box.
    pub async fn get_buildings(&self, bbox: &str) -> EveryMapResult<HereBuildingsResponse> {
        let url = format!("{}/attributes/buildings", self.base_url);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), "json".to_string())];
        params.push(("bbox".to_string(), bbox.to_string()));

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get landmark attributes for a bounding box.
    pub async fn get_landmarks(&self, bbox: &str) -> EveryMapResult<HereLandmarksResponse> {
        let url = format!("{}/attributes/landmarks", self.base_url);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), "json".to_string())];
        params.push(("bbox".to_string(), bbox.to_string()));

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get road attributes by specific feature IDs.
    pub async fn get_road_attributes_by_ids(&self, ids: &[String]) -> EveryMapResult<HereRoadAttributesResponse> {
        let url = format!("{}/attributes/roads", self.base_url);
        let params: Vec<(String, String)> = vec![
            ("format".to_string(), "json".to_string()),
            ("ids".to_string(), ids.join(",")),
        ];

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get speed limits for a bounding area (convenience method).
    pub async fn get_speed_limits(&self, bbox: &str) -> EveryMapResult<HereRoadAttributesResponse> {
        self.get_road_attributes(bbox, Some(vec![
            "LINK_ID".to_string(),
            "SPEED_LIMIT".to_string(),
            "SPEED_LIMITS_BY_DIRECTION".to_string(),
            "FUNCTIONAL_CLASS".to_string(),
            "TRAVEL_DIRECTION".to_string(),
            "NAME".to_string(),
        ])).await
    }
}

/// Convert core `AttributeOptions` to HERE-specific `HereAttributeOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn attribute_options_from_core(opts: &AttributeOptions) -> HereAttributeOptions {
    let mut here_opts = HereAttributeOptions {
        bbox: opts.bbox.clone(),
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("layer").and_then(|v| v.as_str()) {
                here_opts.layer = match v {
                    "roads" => AttributeLayer::Roads,
                    "adminAreas" => AttributeLayer::AdminAreas,
                    "buildings" => AttributeLayer::Buildings,
                    "landmarks" => AttributeLayer::Landmarks,
                    "segments" => AttributeLayer::Segments,
                    _ => AttributeLayer::Roads,
                };
            }
            if let Some(v) = obj.get("format").and_then(|v| v.as_str()) {
                here_opts.format = match v {
                    "geojson" => AttributeFormat::GeoJson,
                    "protobuf" => AttributeFormat::Protobuf,
                    _ => AttributeFormat::Json,
                };
            }
            if let Some(v) = obj.get("ids").and_then(|v| v.as_array()) {
                here_opts.ids = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("srs").and_then(|v| v.as_str()) {
                here_opts.srs = Some(v.to_string());
            }
            if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
                here_opts.political_view = Some(v.to_string());
            }
            if let Some(v) = obj.get("include").and_then(|v| v.as_array()) {
                here_opts.include = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
            if let Some(v) = obj.get("exclude").and_then(|v| v.as_array()) {
                here_opts.exclude = Some(v.iter().filter_map(|i| i.as_str().map(String::from)).collect());
            }
        }
    }

    here_opts
}

#[async_trait]
impl AttributeProvider for HereAttributeProvider {
    async fn get_attributes(&self, options: &AttributeOptions) -> EveryMapResult<AttributeResponse> {
        let here_opts = attribute_options_from_core(options);

        let layer = match &here_opts.layer {
            AttributeLayer::Roads => "roads",
            AttributeLayer::AdminAreas => "adminAreas",
            AttributeLayer::Buildings => "buildings",
            AttributeLayer::Landmarks => "landmarks",
            AttributeLayer::Segments => "segments",
        };

        let format = match &here_opts.format {
            AttributeFormat::Json => "json",
            AttributeFormat::GeoJson => "geojson",
            AttributeFormat::Protobuf => "protobuf",
        };

        let url = format!("{}/attributes/{}", self.base_url, layer);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), format.to_string())];

        if let Some(bbox) = &here_opts.bbox {
            params.push(("bbox".to_string(), bbox.clone()));
        }
        if let Some(ids) = &here_opts.ids {
            params.push(("ids".to_string(), ids.join(",")));
        }
        if let Some(srs) = &here_opts.srs {
            params.push(("srs".to_string(), srs.clone()));
        }
        if let Some(lang) = &here_opts.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(pv) = &here_opts.political_view {
            params.push(("politicalView".to_string(), pv.clone()));
        }
        if let Some(include) = &here_opts.include {
            params.push(("include".to_string(), include.join(",")));
        }
        if let Some(exclude) = &here_opts.exclude {
            params.push(("exclude".to_string(), exclude.join(",")));
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let data: serde_json::Value = self.client.request_json(builder).await?;

        Ok(AttributeResponse { data })
    }
}