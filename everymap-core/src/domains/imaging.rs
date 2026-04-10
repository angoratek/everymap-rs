use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for static map image generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageOptions {
    /// Image format (e.g., "png", "jpg")
    pub format: Option<String>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Provider-specific options (HERE: style, poi, overlay; Google: maptype, markers, path)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Simplified image response from the core trait.
#[derive(Debug, Clone)]
pub struct ImageResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

impl ImageResponse {
    /// Create a new image response from raw bytes.
    pub fn new(data: Vec<u8>, content_type: Option<String>) -> Self {
        Self { data, content_type }
    }
}

#[async_trait]
pub trait MapImageProvider: Send + Sync {
    async fn get_image(&self, center: &Coordinate, zoom: u32, size: (u32, u32), options: &ImageOptions) -> EveryMapResult<ImageResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_options_default() {
        let opts = ImageOptions::default();
        assert!(opts.format.is_none());
        assert!(opts.language.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_image_options_with_fields() {
        let opts = ImageOptions {
            format: Some("png".to_string()),
            language: Some("en".to_string()),
            provider_extra: Some(serde_json::json!({"style": "default"})),
        };
        assert_eq!(opts.format, Some("png".to_string()));
    }

    #[test]
    fn test_image_response_construction() {
        let response = ImageResponse::new(vec![0x89, 0x50, 0x4E, 0x47], Some("image/png".to_string()));
        assert_eq!(response.data.len(), 4);
        assert_eq!(response.content_type, Some("image/png".to_string()));
    }
}