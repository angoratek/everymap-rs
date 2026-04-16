pub mod types;

use async_trait::async_trait;
use everymap_core::domains::imaging::{MapImageProvider, ImageOptions, ImageResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
use std::sync::Arc;

pub use types::*;

const IMAGING_BASE_URL: &str = "https://maps.hereapi.com/mia/v3";

/// Options for HERE Map Image API v3.
#[derive(Debug, Clone, Default)]
pub struct HereImageOptions {
    /// Image format (default: PNG).
    pub format: ImageFormat,
    /// Map style (default: default).
    pub style: Option<String>,
    /// Language for labels (e.g., "en-US").
    pub lang: Option<String>,
    /// Political view for disputed borders (ISO 3166-1 alpha-3).
    pub political_view: Option<String>,
    /// Include POI markers on the map.
    pub poi: Option<String>,
    /// Background color (hex, e.g., "FFFFFF").
    pub bg: Option<String>,
    /// Center marker (adds a marker at the center).
    pub center_marker: Option<bool>,
    /// Overlay data (custom shapes/lines on the map).
    pub overlay: Option<String>,
}

/// Implementation of MapImageProvider for HERE Technologies.
pub struct HereMapImageProvider {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereMapImageProvider {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: IMAGING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

/// Convert core `ImageOptions` to HERE-specific `HereImageOptions`,
/// extracting common fields and parsing `provider_extra` for HERE-specific ones.
fn image_options_from_core(opts: &ImageOptions) -> HereImageOptions {
    let mut here_opts = HereImageOptions {
        lang: opts.language.clone(),
        ..Default::default()
    };

    // Extract HERE-specific options from provider_extra
    if let Some(extra) = &opts.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("format").and_then(|v| v.as_str()) {
                here_opts.format = match v {
                    "jpg" => ImageFormat::Jpg,
                    "gif" => ImageFormat::Gif,
                    "bmp" => ImageFormat::Bmp,
                    "svg" => ImageFormat::Svg,
                    "png8" => ImageFormat::Png8,
                    "png32" => ImageFormat::Png32,
                    _ => ImageFormat::Png,
                };
            }
            if let Some(v) = obj.get("style").and_then(|v| v.as_str()) {
                here_opts.style = Some(v.to_string());
            }
            if let Some(v) = obj.get("political_view").and_then(|v| v.as_str()) {
                here_opts.political_view = Some(v.to_string());
            }
            if let Some(v) = obj.get("poi").and_then(|v| v.as_str()) {
                here_opts.poi = Some(v.to_string());
            }
            if let Some(v) = obj.get("bg").and_then(|v| v.as_str()) {
                here_opts.bg = Some(v.to_string());
            }
            if let Some(v) = obj.get("center_marker").and_then(|v| v.as_bool()) {
                here_opts.center_marker = Some(v);
            }
            if let Some(v) = obj.get("overlay").and_then(|v| v.as_str()) {
                here_opts.overlay = Some(v.to_string());
            }
        }
    }

    here_opts
}

#[async_trait]
impl MapImageProvider for HereMapImageProvider {
    async fn get_image(&self, center: &everymap_core::types::Coordinate, zoom: u32, size: (u32, u32), options: &ImageOptions) -> EveryMapResult<ImageResponse> {
        let here_opts = image_options_from_core(options);

        let format_ext = match &here_opts.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Svg => "svg",
            ImageFormat::Png8 => "png8",
            ImageFormat::Png32 => "png32",
        };

        let url = format!(
            "{}/base/mc/center:{},{};zoom={}/{}x{}/{}",
            self.base_url,
            center.lat,
            center.lng,
            zoom,
            size.0,
            size.1,
            format_ext
        );

        let mut params: Vec<(String, String)> = vec![];

        if let Some(style) = &here_opts.style {
            params.push(("style".to_string(), style.clone()));
        }
        if let Some(lang) = &here_opts.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(pv) = &here_opts.political_view {
            params.push(("politicalView".to_string(), pv.clone()));
        }
        if let Some(poi) = &here_opts.poi {
            params.push(("poi".to_string(), poi.clone()));
        }
        if let Some(bg) = &here_opts.bg {
            params.push(("bg".to_string(), bg.clone()));
        }
        if let Some(overlay) = &here_opts.overlay {
            params.push(("overlay".to_string(), overlay.clone()));
        }

        let builder = self.client.build_request(reqwest::Method::GET, &url)
            .query(&params);

        let response = self.client.request(builder).await?;

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

        let data = response.bytes().await?.to_vec();

        Ok(ImageResponse {
            data,
            content_type,
        })
    }
}