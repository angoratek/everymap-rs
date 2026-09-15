use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for static map image generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageOptions {
    /// Image format (e.g., "png", "jpg")
    pub format: Option<String>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Image width in pixels (overrides the `size` argument when set)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Image height in pixels (overrides the `size` argument when set)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
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
    async fn get_image(
        &self,
        center: &Coordinate,
        zoom: u32,
        size: (u32, u32),
        options: &ImageOptions,
    ) -> EveryMapResult<ImageResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_options_default() {
        let options = ImageOptions::default();
        assert!(options.format.is_none());
        assert!(options.language.is_none());
        assert!(options.width.is_none());
        assert!(options.height.is_none());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_image_options_with_fields() {
        let options = ImageOptions {
            format: Some("png".to_string()),
            language: Some("en".to_string()),
            width: Some(1024),
            height: Some(768),
            provider_extra: Some(serde_json::json!({"style": "default"})),
        };
        assert_eq!(options.format, Some("png".to_string()));
        assert_eq!(options.width, Some(1024));
        assert_eq!(options.height, Some(768));
    }

    #[test]
    fn test_image_options_serde_missing_fields() {
        let back: ImageOptions = serde_json::from_str("{}").unwrap();
        assert!(back.format.is_none());
        assert!(back.language.is_none());
        assert!(back.width.is_none());
        assert!(back.height.is_none());
        assert!(back.provider_extra.is_none());
    }

    #[test]
    fn test_image_response_construction() {
        let response =
            ImageResponse::new(vec![0x89, 0x50, 0x4E, 0x47], Some("image/png".to_string()));
        assert_eq!(response.data.len(), 4);
        assert_eq!(response.content_type, Some("image/png".to_string()));
    }

    // --- ImageOptions serde roundtrip ---

    #[test]
    fn test_image_options_serde_roundtrip() {
        let options = ImageOptions {
            format: Some("jpg".to_string()),
            language: Some("ja".to_string()),
            width: Some(640),
            height: Some(480),
            provider_extra: Some(
                serde_json::json!({"maptype": "satellite", "markers": [{"lat": 35.6762, "lng": 139.6503}]}),
            ),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: ImageOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, Some("jpg".to_string()));
        assert_eq!(back.width, Some(640));
        assert_eq!(back.height, Some(480));
        assert!(back.provider_extra.is_some());
    }

    // --- ImageResponse edge cases ---

    #[test]
    fn test_image_response_empty_data() {
        let response = ImageResponse::new(vec![], None);
        assert!(response.data.is_empty());
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_image_response_no_content_type() {
        let response = ImageResponse::new(vec![0xFF, 0xD8, 0xFF], None);
        assert_eq!(response.data.len(), 3);
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_image_response_jpeg_content_type() {
        let response =
            ImageResponse::new(vec![0xFF, 0xD8, 0xFF, 0xE0], Some("image/jpeg".to_string()));
        assert_eq!(response.content_type, Some("image/jpeg".to_string()));
    }

    #[test]
    fn test_image_response_large_data() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let response = ImageResponse::new(data, Some("image/png".to_string()));
        assert_eq!(response.data.len(), 1024);
    }

    // --- ImageResponse Clone ---

    #[test]
    fn test_image_response_clone() {
        let response = ImageResponse::new(vec![1, 2, 3], Some("image/png".to_string()));
        let cloned = response.clone();
        assert_eq!(cloned.data, response.data);
        assert_eq!(cloned.content_type, response.content_type);
    }
}
