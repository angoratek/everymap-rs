pub mod types;

use async_trait::async_trait;
use everymap_core::domains::imaging::{MapImageProvider, ImageRequest, ImageResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
use std::sync::Arc;

pub use types::*;

const IMAGING_BASE_URL: &str = "https://image.maps.hereapi.com/mia/v3";

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
    client: Arc<HereClient>,
    base_url: String,
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

#[async_trait]
impl MapImageProvider for HereMapImageProvider {
    type Options = HereImageOptions;
    type Response = ImageResponse;

    async fn get_image(&self, req: ImageRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let format_ext = match &req.options.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Svg => "svg",
            ImageFormat::Png8 => "png8",
            ImageFormat::Png32 => "png32",
        };

        let url = format!(
            "{}/maptile/{}/center/{},{}/{}",
            self.base_url,
            req.zoom,
            req.center.lat,
            req.center.lng,
            req.size.0
        );

        let mut params: Vec<(String, String)> = vec![
            ("format".to_string(), format_ext.to_string()),
            ("w".to_string(), req.size.0.to_string()),
            ("h".to_string(), req.size.1.to_string()),
        ];

        if let Some(style) = &req.options.style {
            params.push(("style".to_string(), style.clone()));
        }
        if let Some(lang) = &req.options.lang {
            params.push(("lang".to_string(), lang.clone()));
        }
        if let Some(pv) = &req.options.political_view {
            params.push(("politicalView".to_string(), pv.clone()));
        }
        if let Some(poi) = &req.options.poi {
            params.push(("poi".to_string(), poi.clone()));
        }
        if let Some(bg) = &req.options.bg {
            params.push(("bg".to_string(), bg.clone()));
        }
        if let Some(overlay) = &req.options.overlay {
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