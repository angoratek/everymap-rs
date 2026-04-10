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
}