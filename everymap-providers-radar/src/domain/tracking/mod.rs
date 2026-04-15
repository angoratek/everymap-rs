pub mod types;

use async_trait::async_trait;
use everymap_core::domains::tracking::{
    TripTracker, TripCreateOptions, TripUpdateOptions, TripResult, TripStatus,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use crate::client::RadarClient;
pub use types::*;

const TRIPS_URL: &str = "https://api.radar.io/v1/trips";

/// Implementation of `TripTracker` for Radar.
pub struct RadarTripTracker {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
}

impl RadarTripTracker {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: TRIPS_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

fn parse_trip_status(status: &str) -> TripStatus {
    match status {
        "pending" => TripStatus::Pending,
        "started" => TripStatus::Started,
        "approaching" => TripStatus::Approaching,
        "arrived" => TripStatus::Arrived,
        "completed" => TripStatus::Completed,
        _ => TripStatus::Pending,
    }
}

impl From<RadarTrip> for TripResult {
    fn from(trip: RadarTrip) -> Self {
        let raw = serde_json::to_value(&trip).unwrap_or_default();

        let origin = trip.origin.as_ref().and_then(|o| {
            Coordinate::new(o.latitude, o.longitude).ok()
        });
        let destination = trip.destination.as_ref().and_then(|d| {
            Coordinate::new(d.latitude, d.longitude).ok()
        });

        let status = trip.status.as_deref().map(parse_trip_status);

        Self {
            id: trip.id,
            external_id: trip.external_id,
            status,
            origin,
            destination,
            mode: trip.mode,
            eta: trip.eta,
            metadata: trip.metadata,
            raw: Some(raw),
        }
    }
}

#[async_trait]
impl TripTracker for RadarTripTracker {
    async fn create_trip(&self, options: &TripCreateOptions) -> EveryMapResult<TripResult> {
        let mut body = serde_json::Map::new();

        if let Some(origin) = &options.origin {
            body.insert("origin".to_string(), serde_json::json!({
                "latitude": origin.lat,
                "longitude": origin.lng
            }));
        }
        if let Some(dest) = &options.destination {
            body.insert("destination".to_string(), serde_json::json!({
                "latitude": dest.lat,
                "longitude": dest.lng
            }));
        }
        if let Some(mode) = &options.mode {
            body.insert("mode".to_string(), serde_json::Value::String(mode.clone()));
        }
        if let Some(eid) = &options.external_id {
            body.insert("externalId".to_string(), serde_json::Value::String(eid.clone()));
        }
        if let Some(tag) = &options.tag {
            body.insert("tag".to_string(), serde_json::Value::String(tag.clone()));
        }
        if let Some(metadata) = &options.metadata {
            body.insert("metadata".to_string(), serde_json::to_value(metadata).unwrap_or_default());
        }

        let builder = self.client
            .build_request(reqwest::Method::POST, &self.base_url)
            .json(&serde_json::Value::Object(body));

        let radar_res: RadarTripCreateResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Trip create failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(TripResult::from(radar_res.trip))
    }

    async fn update_trip(&self, options: &TripUpdateOptions) -> EveryMapResult<TripResult> {
        let status_str = options.status.map(|s| match s {
            TripStatus::Pending => "pending",
            TripStatus::Started => "started",
            TripStatus::Approaching => "approaching",
            TripStatus::Arrived => "arrived",
            TripStatus::Completed => "completed",
        }).unwrap_or("pending");

        let url = format!("{}/{}/update", self.base_url, options.trip_id);
        let body = serde_json::json!({ "status": status_str });

        let radar_res: RadarTripCreateResponse = self.client.post_json(&url, &body).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Trip update failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(TripResult::from(radar_res.trip))
    }

    async fn get_trip(&self, trip_id: &str) -> EveryMapResult<TripResult> {
        let url = format!("{}/{}", self.base_url, trip_id);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let radar_res: RadarTripGetResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Get trip failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(TripResult::from(radar_res.trip))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trip() -> RadarTrip {
        RadarTrip {
            id: "trip_123".to_string(),
            external_id: Some("ext_trip_456".to_string()),
            status: Some("started".to_string()),
            origin: Some(crate::domain::types::RadarLocation {
                latitude: 40.7128,
                longitude: -74.0060,
            }),
            destination: Some(crate::domain::types::RadarLocation {
                latitude: 42.3601,
                longitude: -71.0589,
            }),
            mode: Some("car".to_string()),
            eta: Some("2024-06-01T12:00:00Z".to_string()),
            metadata: Some(serde_json::json!({"priority": "high"})),
            tag: Some("delivery".to_string()),
            created_at: Some("2024-06-01T10:00:00Z".to_string()),
            updated_at: Some("2024-06-01T10:30:00Z".to_string()),
        }
    }

    #[test]
    fn test_radar_trip_to_trip_result() {
        let trip = sample_trip();
        let result: TripResult = trip.into();

        assert_eq!(result.id, "trip_123");
        assert_eq!(result.external_id.as_deref(), Some("ext_trip_456"));
        assert!(matches!(result.status, Some(TripStatus::Started)));
        assert_eq!(result.mode.as_deref(), Some("car"));
        assert_eq!(result.eta.as_deref(), Some("2024-06-01T12:00:00Z"));
        assert!(result.origin.is_some());
        assert!((result.origin.as_ref().unwrap().lat - 40.7128).abs() < f64::EPSILON);
        assert!(result.destination.is_some());
        assert!((result.destination.as_ref().unwrap().lat - 42.3601).abs() < f64::EPSILON);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_parse_trip_status_all_variants() {
        assert!(matches!(parse_trip_status("pending"), TripStatus::Pending));
        assert!(matches!(parse_trip_status("started"), TripStatus::Started));
        assert!(matches!(parse_trip_status("approaching"), TripStatus::Approaching));
        assert!(matches!(parse_trip_status("arrived"), TripStatus::Arrived));
        assert!(matches!(parse_trip_status("completed"), TripStatus::Completed));
    }

    #[test]
    fn test_parse_trip_status_unknown_defaults_to_pending() {
        assert!(matches!(parse_trip_status("unknown"), TripStatus::Pending));
        assert!(matches!(parse_trip_status(""), TripStatus::Pending));
    }

    #[test]
    fn test_radar_trip_empty_fields() {
        let trip = RadarTrip {
            id: String::new(),
            external_id: None,
            status: None,
            origin: None,
            destination: None,
            mode: None,
            eta: None,
            metadata: None,
            tag: None,
            created_at: None,
            updated_at: None,
        };
        let result: TripResult = trip.into();

        assert!(result.id.is_empty());
        assert!(result.external_id.is_none());
        assert!(result.status.is_none());
        assert!(result.origin.is_none());
        assert!(result.destination.is_none());
        assert!(result.mode.is_none());
        assert!(result.eta.is_none());
    }
}