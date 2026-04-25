pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::tiling::{TileOptions, TileProvider, TileResponse};
use everymap_core::error::EveryMapResult;
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
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
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

/// Convert core `TileOptions` to HERE-specific `HereTileOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn tile_options_from_core(opts: &TileOptions) -> HereTileOptions {
    let mut here_opts = HereTileOptions::default();

    // Common fields
    if let Some(format) = &opts.format {
        here_opts.format = match format.as_str() {
            "omv" | "protobuf" => TileFormat::OmnichannelVector,
            "pbf" => TileFormat::Protobuf,
            _ => TileFormat::OmnichannelVector,
        };
    }

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("layer").and_then(|v| v.as_str()) {
                here_opts.layer = match v {
                    "base" => TileLayer::Base,
                    "core" => TileLayer::Core,
                    "hybrid" => TileLayer::Hybrid,
                    "mapbox" => TileLayer::Mapbox,
                    _ => TileLayer::Base,
                };
            }
            if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
                here_opts.political_view = Some(v.to_string());
            }
        }
    }

    here_opts
}

#[async_trait]
impl TileProvider for HereTileProvider {
    async fn get_tile(
        &self,
        z: u32,
        x: u32,
        y: u32,
        options: &TileOptions,
    ) -> EveryMapResult<TileResponse> {
        let here_opts = tile_options_from_core(options);

        let layer = match &here_opts.layer {
            TileLayer::Mapbox => "mapbox",
            TileLayer::Base => "base",
            TileLayer::Core => "core",
            TileLayer::Hybrid => "hybrid",
        };

        let format_ext = match &here_opts.format {
            TileFormat::OmnichannelVector => "omv",
            TileFormat::Protobuf => "pbf",
        };

        let url = format!(
            "{}/vectortiles/{}/mc/{}/{}/{}/{}",
            self.base_url, layer, z, x, y, format_ext
        );

        let mut builder = self.client.build_request(reqwest::Method::GET, &url);

        if let Some(political_view) = &here_opts.political_view {
            builder = builder.query(&[("politicalView", political_view)]);
        }

        let response = self.client.request(builder).await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                // Strip any charset suffix
                s.split(';').next().unwrap_or(s).trim().to_string()
            });

        let data = response.bytes().await?.to_vec();

        Ok(TileResponse { data, content_type })
    }
}
