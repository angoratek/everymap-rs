use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for searching geofences near a location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeofenceOptions {
    /// Center point for the search
    pub near: Option<Coordinate>,
    /// Search radius in meters
    pub radius: Option<f64>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Whether to include geometry in results
    pub include_geometry: Option<bool>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Options for creating or updating a geofence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeofenceCreateOptions {
    /// Tag for grouping geofences
    pub tag: Option<String>,
    /// External ID for linking to external systems
    pub external_id: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Type of geofence geometry
    pub geofence_type: Option<GeofenceType>,
    /// GeoJSON geometry for polygon/isochrone geofences
    pub geometry: Option<serde_json::Value>,
    /// Center coordinate for circle geofences
    pub center: Option<Coordinate>,
    /// Radius in meters for circle geofences
    pub radius: Option<f64>,
    /// Arbitrary metadata key-value pairs
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Whether the geofence is active
    pub enabled: Option<bool>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Geofence geometry type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GeofenceType {
    Circle,
    Polygon,
    Isochrone,
}

/// A unified geofence result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofenceResult {
    /// Unique identifier for this geofence
    pub id: String,
    /// Tag for grouping
    pub tag: Option<String>,
    /// External ID for linking to external systems
    pub external_id: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Type of geofence geometry
    pub geofence_type: Option<GeofenceType>,
    /// Center coordinate of the geofence
    pub geometry_center: Option<Coordinate>,
    /// Arbitrary metadata
    pub metadata: Option<serde_json::Value>,
    /// Whether the geofence is active
    pub enabled: Option<bool>,
    /// Provider-specific raw data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// Response from a geofence search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofenceResponse {
    pub geofences: Vec<GeofenceResult>,
}

/// Core trait for geofencing providers.
///
/// Provides CRUD operations for geofences — virtual boundaries that trigger
/// events when users enter or exit them. Radar is the primary provider;
/// other providers return `UnsupportedDomain`.
#[async_trait]
pub trait GeofenceProvider: Send + Sync {
    /// Search for geofences near a location.
    async fn search_geofences(&self, options: &GeofenceOptions) -> EveryMapResult<GeofenceResponse>;

    /// Create a new geofence.
    async fn create_geofence(&self, options: &GeofenceCreateOptions) -> EveryMapResult<GeofenceResult>;

    /// Get a geofence by ID.
    async fn get_geofence(&self, id: &str) -> EveryMapResult<GeofenceResult>;

    /// Delete a geofence by ID.
    async fn delete_geofence(&self, id: &str) -> EveryMapResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geofence_options_default() {
        let opts = GeofenceOptions::default();
        assert!(opts.near.is_none());
        assert!(opts.radius.is_none());
        assert!(opts.tags.is_empty());
        assert!(opts.limit.is_none());
        assert!(opts.include_geometry.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_geofence_create_options_default() {
        let opts = GeofenceCreateOptions::default();
        assert!(opts.tag.is_none());
        assert!(opts.external_id.is_none());
        assert!(opts.description.is_none());
        assert!(opts.geofence_type.is_none());
        assert!(opts.geometry.is_none());
        assert!(opts.center.is_none());
        assert!(opts.radius.is_none());
        assert!(opts.metadata.is_none());
        assert!(opts.enabled.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_geofence_type_serde() {
        let gt = GeofenceType::Circle;
        let json = serde_json::to_string(&gt).unwrap();
        assert!(json.contains("circle") || json.contains("Circle"));
    }

    #[test]
    fn test_geofence_type_all_variants_serde() {
        let variants = [GeofenceType::Circle, GeofenceType::Polygon, GeofenceType::Isochrone];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: GeofenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    #[test]
    fn test_geofence_result_construction() {
        let result = GeofenceResult {
            id: "gf_1".to_string(),
            tag: Some("test".to_string()),
            external_id: None,
            description: Some("A test geofence".to_string()),
            geofence_type: Some(GeofenceType::Circle),
            geometry_center: None,
            metadata: None,
            enabled: Some(true),
            raw: None,
        };
        assert_eq!(result.id, "gf_1");
        assert_eq!(result.tag.as_deref(), Some("test"));
        assert!(result.enabled.unwrap());
        assert!(result.raw.is_none());
    }

    #[test]
    fn test_geofence_options_with_tags() {
        let opts = GeofenceOptions {
            near: Some(Coordinate::ORIGIN),
            radius: Some(1000.0),
            tags: vec!["store".to_string(), "warehouse".to_string()],
            limit: Some(10),
            include_geometry: Some(true),
            provider_extra: None,
        };
        assert_eq!(opts.tags.len(), 2);
        assert_eq!(opts.radius.unwrap(), 1000.0);
        assert_eq!(opts.limit.unwrap(), 10);
    }

    #[test]
    fn test_geofence_create_options_with_all_fields() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("level".to_string(), serde_json::json!("gold"));
        let opts = GeofenceCreateOptions {
            tag: Some("store".to_string()),
            external_id: Some("ext_1".to_string()),
            description: Some("A store zone".to_string()),
            geofence_type: Some(GeofenceType::Circle),
            geometry: None,
            center: Some(Coordinate::ORIGIN),
            radius: Some(500.0),
            metadata: Some(metadata),
            enabled: Some(true),
            provider_extra: None,
        };
        assert!(opts.center.is_some());
        assert_eq!(opts.radius.unwrap(), 500.0);
        assert!(opts.metadata.unwrap().contains_key("level"));
    }
}