pub mod types;

use async_trait::async_trait;
use everymap_core::domains::attributes::{AttributeProvider, AttributeRequest, AttributeResponse};
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
}

#[async_trait]
impl AttributeProvider for HereAttributeProvider {
    type Options = HereAttributeOptions;
    type Response = AttributeResponse;

    async fn get_attributes(&self, req: AttributeRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let layer = match &req.options.layer {
            AttributeLayer::Roads => "roads",
            AttributeLayer::AdminAreas => "adminAreas",
            AttributeLayer::Buildings => "buildings",
            AttributeLayer::Landmarks => "landmarks",
            AttributeLayer::Segments => "segments",
        };

        let format = match &req.options.format {
            AttributeFormat::Json => "json",
            AttributeFormat::GeoJson => "geojson",
            AttributeFormat::Protobuf => "protobuf",
        };

        let url = format!("{}/attributes/{}", self.base_url, layer);
        let mut params: Vec<(String, String)> = vec![("format".to_string(), format.to_string())];

        if let Some(bbox) = &req.options.bbox {
            params.push(("bbox".to_string(), bbox.clone()));
        }
        if let Some(ids) = &req.options.ids {
            params.push(("ids".to_string(), ids.join(",")));
        }
        if let Some(srs) = &req.options.srs {
            params.push(("srs".to_string(), srs.clone()));
        }
        if let Some(lang) = &req.options.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(pv) = &req.options.political_view {
            params.push(("politicalView".to_string(), pv.clone()));
        }
        if let Some(include) = &req.options.include {
            params.push(("include".to_string(), include.join(",")));
        }
        if let Some(exclude) = &req.options.exclude {
            params.push(("exclude".to_string(), exclude.join(",")));
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;
        let data: serde_json::Value = response.json().await?;

        Ok(AttributeResponse { data })
    }
}