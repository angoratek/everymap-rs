use async_trait::async_trait;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for map tile retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileOptions {
    /// Response format (e.g., "protobuf", "json")
    pub format: Option<String>,
    /// Provider-specific options (HERE: layer, political_view; Mapbox: style_id, access_token)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Simplified tile response from the core trait.
#[derive(Debug, Clone)]
pub struct TileResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

impl TileResponse {
    /// Create a new tile response from raw bytes.
    pub fn new(data: Vec<u8>, content_type: Option<String>) -> Self {
        Self { data, content_type }
    }
}

#[async_trait]
pub trait TileProvider: Send + Sync {
    async fn get_tile(&self, z: u32, x: u32, y: u32, options: &TileOptions) -> EveryMapResult<TileResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_options_default() {
        let opts = TileOptions::default();
        assert!(opts.format.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_tile_options_with_fields() {
        let opts = TileOptions {
            format: Some("protobuf".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "base"})),
        };
        assert_eq!(opts.format, Some("protobuf".to_string()));
    }

    #[test]
    fn test_tile_response_construction() {
        let response = TileResponse::new(vec![1, 2, 3], Some("application/x-protobuf".to_string()));
        assert_eq!(response.data.len(), 3);
        assert_eq!(response.content_type, Some("application/x-protobuf".to_string()));
    }

    // --- TileOptions serde roundtrip ---

    #[test]
    fn test_tile_options_serde_roundtrip() {
        let opts = TileOptions {
            format: Some("json".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "base", "political_view": "ARG"})),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: TileOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, Some("json".to_string()));
        assert!(back.provider_extra.is_some());
    }

    // --- TileResponse edge cases ---

    #[test]
    fn test_tile_response_empty_data() {
        let response = TileResponse::new(vec![], None);
        assert!(response.data.is_empty());
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_tile_response_no_content_type() {
        let response = TileResponse::new(vec![0x1A, 0x2B, 0x3C, 0x4D], None);
        assert_eq!(response.data.len(), 4);
        assert!(response.content_type.is_none());
    }

    #[test]
    fn test_tile_response_json_content_type() {
        let response = TileResponse::new(b"{\"tile\":1}".to_vec(), Some("application/json".to_string()));
        assert_eq!(response.content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_tile_response_large_data() {
        let data: Vec<u8> = (0..8192).map(|i| (i % 256) as u8).collect();
        let response = TileResponse::new(data, Some("application/x-protobuf".to_string()));
        assert_eq!(response.data.len(), 8192);
    }

    // --- TileResponse Clone ---

    #[test]
    fn test_tile_response_clone() {
        let response = TileResponse::new(vec![1, 2, 3], Some("test/type".to_string()));
        let cloned = response.clone();
        assert_eq!(cloned.data, response.data);
        assert_eq!(cloned.content_type, response.content_type);
    }

    // --- TileResponse Debug ---

    #[test]
    fn test_tile_response_debug() {
        let response = TileResponse::new(vec![1], Some("application/json".to_string()));
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("TileResponse"));
    }
}