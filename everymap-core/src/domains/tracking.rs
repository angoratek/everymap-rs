use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for creating a trip.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TripCreateOptions {
    /// Origin coordinate
    pub origin: Option<Coordinate>,
    /// Destination coordinate
    pub destination: Option<Coordinate>,
    /// Travel mode (e.g., "car", "foot", "bike")
    pub mode: Option<String>,
    /// External ID for linking to external systems
    pub external_id: Option<String>,
    /// Arbitrary metadata key-value pairs
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Tag for grouping trips
    pub tag: Option<String>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Options for updating a trip's status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TripUpdateOptions {
    /// Trip ID to update
    pub trip_id: String,
    /// New status for the trip
    pub status: Option<TripStatus>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Trip status progression:
/// `pending` → `started` → `approaching` → `arrived` → `completed`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TripStatus {
    Pending,
    Started,
    Approaching,
    Arrived,
    Completed,
}

/// A unified trip result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripResult {
    /// Unique identifier for this trip
    pub id: String,
    /// External ID for linking to external systems
    pub external_id: Option<String>,
    /// Current status of the trip
    pub status: Option<TripStatus>,
    /// Origin coordinate
    pub origin: Option<Coordinate>,
    /// Destination coordinate
    pub destination: Option<Coordinate>,
    /// Travel mode
    pub mode: Option<String>,
    /// Estimated time of arrival (ISO 8601)
    pub eta: Option<String>,
    /// Arbitrary metadata
    pub metadata: Option<serde_json::Value>,
    /// Provider-specific raw data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// Core trait for trip tracking providers.
///
/// Provides real-time trip tracking with live ETAs and arrival detection.
/// Radar is the primary provider; other providers return `UnsupportedDomain`.
#[async_trait]
pub trait TripTracker: Send + Sync {
    /// Create a new trip.
    async fn create_trip(&self, options: &TripCreateOptions) -> EveryMapResult<TripResult>;

    /// Update a trip's status.
    async fn update_trip(&self, options: &TripUpdateOptions) -> EveryMapResult<TripResult>;

    /// Get a trip by ID.
    async fn get_trip(&self, trip_id: &str) -> EveryMapResult<TripResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trip_create_options_default() {
        let opts = TripCreateOptions::default();
        assert!(opts.origin.is_none());
        assert!(opts.destination.is_none());
        assert!(opts.mode.is_none());
        assert!(opts.external_id.is_none());
        assert!(opts.metadata.is_none());
        assert!(opts.tag.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_trip_status_serde() {
        let status = TripStatus::Approaching;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("approaching") || json.contains("Approaching"));
    }

    #[test]
    fn test_trip_status_progression() {
        let statuses = [
            TripStatus::Pending,
            TripStatus::Started,
            TripStatus::Approaching,
            TripStatus::Arrived,
            TripStatus::Completed,
        ];
        // Just verify all variants exist and are distinct
        for i in 0..statuses.len() {
            for j in 0..statuses.len() {
                if i != j {
                    assert_ne!(statuses[i], statuses[j]);
                }
            }
        }
    }

    #[test]
    fn test_trip_status_all_serde_roundtrip() {
        let statuses = [
            TripStatus::Pending,
            TripStatus::Started,
            TripStatus::Approaching,
            TripStatus::Arrived,
            TripStatus::Completed,
        ];
        for s in &statuses {
            let json = serde_json::to_string(s).unwrap();
            let back: TripStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*s, back);
        }
    }

    #[test]
    fn test_trip_result_construction() {
        let result = TripResult {
            id: "trip_1".to_string(),
            external_id: Some("ext_1".to_string()),
            status: Some(TripStatus::Started),
            origin: Some(Coordinate::ORIGIN),
            destination: None,
            mode: Some("car".to_string()),
            eta: Some("2024-06-01T12:00:00Z".to_string()),
            metadata: None,
            raw: None,
        };
        assert_eq!(result.id, "trip_1");
        assert!(matches!(result.status, Some(TripStatus::Started)));
        assert_eq!(result.mode.as_deref(), Some("car"));
    }

    #[test]
    fn test_trip_update_options() {
        let opts = TripUpdateOptions {
            trip_id: "trip_1".to_string(),
            status: Some(TripStatus::Arrived),
            provider_extra: None,
        };
        assert_eq!(opts.trip_id, "trip_1");
        assert!(matches!(opts.status, Some(TripStatus::Arrived)));
    }

    #[test]
    fn test_trip_create_options_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("priority".to_string(), serde_json::json!("high"));
        let opts = TripCreateOptions {
            origin: Some(Coordinate::ORIGIN),
            destination: None,
            mode: Some("car".to_string()),
            external_id: Some("ext_trip".to_string()),
            metadata: Some(metadata),
            tag: Some("delivery".to_string()),
            provider_extra: None,
        };
        assert!(opts.metadata.unwrap().contains_key("priority"));
        assert_eq!(opts.tag.as_deref(), Some("delivery"));
    }
}