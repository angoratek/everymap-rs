pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider, AttributeResponse};
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

pub use types::*;

const ATTRIBUTES_BASE_URL: &str = "https://smap.hereapi.com/v8";

/// Options for HERE Map Attributes API v8.
#[derive(Debug, Clone, Default)]
pub struct HereAttributeOptions {
    /// The attribute layers to query (e.g., "ROAD_GEOM_FCn", "SPEED_LIMITS_FCn").
    pub layers: Option<Vec<String>>,
    /// Spatial filter using the `in` parameter.
    /// Format: "bbox:lat1,lon1,lat2,lon2" or "proximity:lat,lon;r=radius" or "tile:tileId1,tileId2".
    pub in_filter: Option<String>,
    /// Specific feature IDs to retrieve (uses `ids` parameter).
    pub ids: Option<Vec<String>>,
    /// Spatial reference system (e.g., "EPSG:4326").
    pub srs: Option<String>,
    /// Language for localized strings.
    pub lang: Option<String>,
    /// Political view for disputed borders.
    pub political_view: Option<String>,
}

/// Implementation of AttributeProvider for HERE Technologies.
pub struct HereAttributeProvider {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
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

    /// Get map attributes for a spatial filter.
    /// This is the primary method that maps to GET /v8/maps/attributes.
    pub async fn get_map_attributes(
        &self,
        options: &HereAttributeOptions,
    ) -> EveryMapResult<serde_json::Value> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];

        if let Some(in_filter) = &options.in_filter {
            params.push(("in".to_string(), in_filter.clone()));
        }
        if let Some(layers) = &options.layers {
            params.push(("layers".to_string(), layers.join(",")));
        }
        if let Some(ids) = &options.ids {
            params.push(("ids".to_string(), ids.join(",")));
        }
        if let Some(srs) = &options.srs {
            params.push(("srs".to_string(), srs.clone()));
        }
        if let Some(lang) = &options.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(political_view) = &options.political_view {
            params.push(("politicalView".to_string(), political_view.clone()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        self.client.request_json(builder).await
    }

    /// Get road attributes for a bounding box.
    /// Convenience method that returns typed road attribute data.
    /// The bbox can be "lat1,lon1,lat2,lon2" or "lat1,lon1;lat2,lon2".
    pub async fn get_road_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<HereRoadAttributesResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];
        params.push(("in".to_string(), format!("bbox:{}", bbox.replace(';', ","))));
        if let Some(inc) = includes {
            params.push(("layers".to_string(), inc.join(",")));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get segment (topology) attributes for a bounding box.
    /// The bbox can be "lat1,lon1,lat2,lon2" or "lat1,lon1;lat2,lon2".
    pub async fn get_segment_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<HereSegmentAttributesResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];
        params.push(("in".to_string(), format!("bbox:{}", bbox.replace(';', ","))));
        if let Some(inc) = includes {
            params.push(("layers".to_string(), inc.join(",")));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get administrative area attributes for a bounding box.
    /// The bbox can be "lat1,lon1,lat2,lon2" or "lat1,lon1;lat2,lon2".
    pub async fn get_admin_areas(&self, bbox: &str) -> EveryMapResult<HereAdminAreasResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];
        params.push(("in".to_string(), format!("bbox:{}", bbox.replace(';', ","))));

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get building attributes for a bounding box.
    /// The bbox can be "lat1,lon1,lat2,lon2" or "lat1,lon1;lat2,lon2".
    pub async fn get_buildings(&self, bbox: &str) -> EveryMapResult<HereBuildingsResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];
        params.push(("in".to_string(), format!("bbox:{}", bbox.replace(';', ","))));

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get landmark attributes for a bounding box.
    /// The bbox can be "lat1,lon1,lat2,lon2" or "lat1,lon1;lat2,lon2".
    pub async fn get_landmarks(&self, bbox: &str) -> EveryMapResult<HereLandmarksResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let mut params: Vec<(String, String)> = vec![];
        params.push(("in".to_string(), format!("bbox:{}", bbox.replace(';', ","))));

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get road attributes by specific feature IDs.
    pub async fn get_road_attributes_by_ids(
        &self,
        ids: &[String],
    ) -> EveryMapResult<HereRoadAttributesResponse> {
        let url = format!("{}/maps/attributes", self.base_url);
        let params: Vec<(String, String)> = vec![("ids".to_string(), ids.join(","))];

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);
        self.client.request_json(builder).await
    }

    /// Get speed limits for a bounding area (convenience method).
    /// The bbox format should be "lat1,lon1,lat2,lon2" (comma-separated).
    pub async fn get_speed_limits(&self, bbox: &str) -> EveryMapResult<HereRoadAttributesResponse> {
        // Convert semicolon-separated bbox to comma-separated if needed
        let normalized_bbox = bbox.replace(';', ",");
        self.get_road_attributes(&normalized_bbox, Some(vec!["SPEED_LIMITS_FCn".to_string()]))
            .await
    }
}

/// Convert core `AttributeOptions` to HERE-specific `HereAttributeOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn attribute_options_from_core(options: &AttributeOptions) -> HereAttributeOptions {
    let mut here_options = HereAttributeOptions {
        lang: options.language.clone(),
        ..Default::default()
    };

    // Convert bbox to the HERE `in` filter format
    if let Some(bbox) = &options.bbox {
        // If bbox already starts with "bbox:", "proximity:", or "tile:", use as-is
        if bbox.starts_with("bbox:") || bbox.starts_with("proximity:") || bbox.starts_with("tile:")
        {
            here_options.in_filter = Some(bbox.clone());
        } else {
            // Convert "lat1,lon1;lat2,lon2" or "south,west;north,east" to "bbox:..."
            here_options.in_filter = Some(format!("bbox:{}", bbox.replace(';', ",")));
        }
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("layers").and_then(|v| v.as_array()) {
                here_options.layers = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("in_filter").and_then(|v| v.as_str()) {
                here_options.in_filter = Some(v.to_string());
            }
            if let Some(v) = obj.get("ids").and_then(|v| v.as_array()) {
                here_options.ids = Some(
                    v.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect(),
                );
            }
            if let Some(v) = obj.get("srs").and_then(|v| v.as_str()) {
                here_options.srs = Some(v.to_string());
            }
            if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
                here_options.political_view = Some(v.to_string());
            }
        }
    }

    here_options
}

#[async_trait]
impl AttributeProvider for HereAttributeProvider {
    async fn get_attributes(
        &self,
        options: &AttributeOptions,
    ) -> EveryMapResult<AttributeResponse> {
        let here_options = attribute_options_from_core(options);

        let data = self.get_map_attributes(&here_options).await?;

        Ok(AttributeResponse { data })
    }
}
