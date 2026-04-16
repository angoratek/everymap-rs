use async_trait::async_trait;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for map attribute queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributeOptions {
    /// Bounding box for the attribute query (format: "south,west,north,east")
    pub bbox: Option<String>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Provider-specific options (HERE: layer, format, ids, srs, include, exclude)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Simplified attribute response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeResponse {
    /// The attribute data as JSON
    pub data: serde_json::Value,
}

#[async_trait]
pub trait AttributeProvider: Send + Sync {
    async fn get_attributes(&self, options: &AttributeOptions) -> EveryMapResult<AttributeResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_options_default() {
        let opts = AttributeOptions::default();
        assert!(opts.bbox.is_none());
        assert!(opts.language.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_attribute_options_with_fields() {
        let opts = AttributeOptions {
            bbox: Some("52.0,13.0,52.5,13.5".to_string()),
            language: Some("en".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "speed_limit"})),
        };
        assert_eq!(opts.bbox, Some("52.0,13.0,52.5,13.5".to_string()));
        assert_eq!(opts.language, Some("en".to_string()));
    }

    #[test]
    fn test_attribute_response_construction() {
        let response = AttributeResponse {
            data: serde_json::json!({"features": []}),
        };
        assert!(response.data.is_object());
    }

    // --- AttributeResponse serde roundtrip ---

    #[test]
    fn test_attribute_response_serde_roundtrip() {
        let response = AttributeResponse {
            data: serde_json::json!({
                "features": [
                    {"id": 1, "speed_limit": 50, "road_name": "Main St"},
                    {"id": 2, "speed_limit": 30, "road_name": "School Zone"}
                ],
                "type": "FeatureCollection"
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: AttributeResponse = serde_json::from_str(&json).unwrap();
        assert!(back.data.is_object());
        assert_eq!(back.data["features"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_attribute_response_empty_object() {
        let response = AttributeResponse {
            data: serde_json::json!({}),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: AttributeResponse = serde_json::from_str(&json).unwrap();
        assert!(back.data.is_object());
        assert!(back.data.as_object().unwrap().is_empty());
    }

    // --- AttributeOptions serde roundtrip ---

    #[test]
    fn test_attribute_options_serde_roundtrip() {
        let opts = AttributeOptions {
            bbox: Some("52.0,13.0,52.5,13.5".to_string()),
            language: Some("en".to_string()),
            provider_extra: Some(serde_json::json!({"layer": "speed_limit", "format": "json"})),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: AttributeOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.bbox, Some("52.0,13.0,52.5,13.5".to_string()));
        assert!(back.provider_extra.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_attribute_response_with_array_data() {
        let response = AttributeResponse {
            data: serde_json::json!([1, 2, 3]),
        };
        assert!(response.data.is_array());
    }

    #[test]
    fn test_attribute_response_with_null_data() {
        let response = AttributeResponse {
            data: serde_json::Value::Null,
        };
        assert!(response.data.is_null());
    }
}