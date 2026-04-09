pub mod types;

use async_trait::async_trait;
use everymap_core::domains::tiling::{TileProvider, TileRequest, TileResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
use std::sync::Arc;

pub use types::*;

const TILING_BASE_URL: &str = "https://vector.hereapi.com/v2";

/// Options for HERE Vector Tile API v2.
#[derive(Debug, Clone, Default)]
pub struct HereTileOptions {
    /// The tile layer to request.
    pub layer: TileLayer,
    /// Tile format (default: OmnichannelVector / .omv).
    pub format: TileFormat,
    /// Political view for disputed borders (ISO 3166-1 alpha-3 country code).
    pub political_view: Option<String>,
}

/// Implementation of TileProvider for HERE Technologies.
pub struct HereTileProvider {
    client: Arc<HereClient>,
    base_url: String,
}

impl HereTileProvider {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: TILING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait]
impl TileProvider for HereTileProvider {
    type Options = HereTileOptions;
    type Response = TileResponse;

    async fn get_tile(&self, req: TileRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let layer = match &req.options.layer {
            TileLayer::Mapbox => "mapbox",
            TileLayer::Base => "base",
            TileLayer::Core => "core",
            TileLayer::Hybrid => "hybrid",
        };

        let format_ext = match &req.options.format {
            TileFormat::OmnichannelVector => "omv",
            TileFormat::Protobuf => "pbf",
        };

        let url = format!(
            "{}/vectortiles/{}/{}/{}/{}.{}",
            self.base_url, layer, req.z, req.x, req.y, format_ext
        );

        let mut builder = self.client.build_request(reqwest::Method::GET, &url);

        if let Some(pv) = &req.options.political_view {
            builder = builder.query(&[("politicalView", pv)]);
        }

        let response = self.client.request(builder).await?;

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                // Strip any charset suffix
                s.split(';').next().unwrap_or(s).trim().to_string()
            });

        let data = response.bytes().await?
            .to_vec();

        Ok(TileResponse {
            data,
            content_type,
        })
    }
}