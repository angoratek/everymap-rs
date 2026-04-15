pub mod types;

use async_trait::async_trait;
use everymap_core::domains::geofencing::{
    GeofenceProvider, GeofenceOptions, GeofenceCreateOptions, GeofenceResponse, GeofenceResult, GeofenceType,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use crate::client::RadarClient;
pub use types::*;

const GEOFENCE_SEARCH_URL: &str = "https://api.radar.io/v1/search/geofences";
const GEOFENCE_LIST_URL: &str = "https://api.radar.io/v1/geofences";

/// Implementation of `GeofenceProvider` for Radar.
pub struct RadarGeofenceProvider {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) search_url: String,
    pub(crate) list_url: String,
}

impl RadarGeofenceProvider {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            search_url: GEOFENCE_SEARCH_URL.to_string(),
            list_url: GEOFENCE_LIST_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, search_url: String, list_url: String) -> Self {
        Self { client, search_url, list_url }
    }
}

impl From<RadarGeofence> for GeofenceResult {
    fn from(gf: RadarGeofence) -> Self {
        let raw = serde_json::to_value(&gf).unwrap_or_default();

        let geometry_center = gf.geometry_center.as_ref().map(|c| {
            Coordinate::new(c.latitude, c.longitude).unwrap_or(Coordinate::ORIGIN)
        });

        let geofence_type = gf.gf_type.as_deref().map(|t| match t {
            "circle" => GeofenceType::Circle,
            "polygon" => GeofenceType::Polygon,
            "isochrone" => GeofenceType::Isochrone,
            _ => GeofenceType::Circle,
        });

        Self {
            id: gf.id,
            tag: gf.tag,
            external_id: gf.external_id,
            description: gf.description,
            geofence_type,
            geometry_center,
            metadata: gf.metadata,
            enabled: gf.enabled,
            raw: Some(raw),
        }
    }
}

#[async_trait]
impl GeofenceProvider for RadarGeofenceProvider {
    async fn search_geofences(&self, options: &GeofenceOptions) -> EveryMapResult<GeofenceResponse> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(near) = &options.near {
            params.push(("near", format!("{},{}", near.lat, near.lng)));
        }
        if let Some(radius) = options.radius {
            params.push(("radius", radius.to_string()));
        }
        if !options.tags.is_empty() {
            params.push(("tags", options.tags.join(",")));
        }
        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(ig) = options.include_geometry {
            params.push(("includeGeometry", ig.to_string()));
        }

        let builder = self.client
            .build_request(reqwest::Method::GET, &self.search_url)
            .query(&params);

        let radar_res: RadarGeofenceSearchResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Geofence search failed with status {}", radar_res.meta.code),
            ));
        }

        let geofences: Vec<GeofenceResult> = radar_res.geofences.into_iter()
            .map(GeofenceResult::from)
            .collect();

        Ok(GeofenceResponse { geofences })
    }

    async fn create_geofence(&self, options: &GeofenceCreateOptions) -> EveryMapResult<GeofenceResult> {
        let mut body = serde_json::Map::new();

        if let Some(tag) = &options.tag {
            body.insert("tag".to_string(), serde_json::Value::String(tag.clone()));
        }
        if let Some(eid) = &options.external_id {
            body.insert("externalId".to_string(), serde_json::Value::String(eid.clone()));
        }
        if let Some(desc) = &options.description {
            body.insert("description".to_string(), serde_json::Value::String(desc.clone()));
        }
        if let Some(gt) = &options.geofence_type {
            let type_str = match gt {
                GeofenceType::Circle => "circle",
                GeofenceType::Polygon => "polygon",
                GeofenceType::Isochrone => "isochrone",
            };
            body.insert("type".to_string(), serde_json::Value::String(type_str.to_string()));
        }
        if let Some(center) = &options.center {
            body.insert("coordinates".to_string(), serde_json::json!([center.lng, center.lat]));
        }
        if let Some(radius) = options.radius {
            body.insert("radius".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(radius).unwrap_or(serde_json::Number::from(0))));
        }
        if let Some(enabled) = options.enabled {
            body.insert("enabled".to_string(), serde_json::Value::Bool(enabled));
        }
        if let Some(metadata) = &options.metadata {
            body.insert("metadata".to_string(), serde_json::to_value(metadata).unwrap_or_default());
        }
        if let Some(geometry) = &options.geometry {
            body.insert("geometry".to_string(), geometry.clone());
        }

        let url = if let (Some(tag), Some(eid)) = (&options.tag, &options.external_id) {
            format!("{}/{}/{}", self.list_url, tag, eid)
        } else {
            self.list_url.clone()
        };

        let builder = self.client
            .build_request(reqwest::Method::PUT, &url)
            .json(&serde_json::Value::Object(body));

        let radar_res: RadarGeofenceCreateResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Geofence create failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(GeofenceResult::from(radar_res.geofence))
    }

    async fn get_geofence(&self, id: &str) -> EveryMapResult<GeofenceResult> {
        let url = format!("{}/{}", self.list_url, id);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let radar_res: RadarGeofenceGetResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Get geofence failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(GeofenceResult::from(radar_res.geofence))
    }

    async fn delete_geofence(&self, id: &str) -> EveryMapResult<()> {
        let url = format!("{}/{}", self.list_url, id);
        let builder = self.client.build_request(reqwest::Method::DELETE, &url);

        let response = self.client.request(builder).await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(EveryMapError::http_with_body(
                status.as_u16(),
                "Delete geofence failed",
                body,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_geofence() -> RadarGeofence {
        RadarGeofence {
            id: "gf_123".to_string(),
            live: Some(true),
            tag: Some("store".to_string()),
            external_id: Some("ext_456".to_string()),
            description: Some("NYC store zone".to_string()),
            gf_type: Some("circle".to_string()),
            geometry_center: Some(crate::domain::types::RadarLocation {
                latitude: 40.7128,
                longitude: -74.0060,
            }),
            metadata: Some(serde_json::json!({"level": "gold"})),
            enabled: Some(true),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            geometry: None,
            radius: Some(500.0),
        }
    }

    #[test]
    fn test_radar_geofence_to_geofence_result_circle() {
        let gf = sample_geofence();
        let result: GeofenceResult = gf.into();

        assert_eq!(result.id, "gf_123");
        assert_eq!(result.tag.as_deref(), Some("store"));
        assert_eq!(result.external_id.as_deref(), Some("ext_456"));
        assert_eq!(result.description.as_deref(), Some("NYC store zone"));
        assert!(matches!(result.geofence_type, Some(GeofenceType::Circle)));
        assert!(result.enabled == Some(true));
        assert!(result.geometry_center.is_some());
        assert!((result.geometry_center.as_ref().unwrap().lat - 40.7128).abs() < f64::EPSILON);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_radar_geofence_to_geofence_result_polygon() {
        let mut gf = sample_geofence();
        gf.gf_type = Some("polygon".to_string());
        let result: GeofenceResult = gf.into();

        assert!(matches!(result.geofence_type, Some(GeofenceType::Polygon)));
    }

    #[test]
    fn test_radar_geofence_to_geofence_result_isochrone() {
        let mut gf = sample_geofence();
        gf.gf_type = Some("isochrone".to_string());
        let result: GeofenceResult = gf.into();

        assert!(matches!(result.geofence_type, Some(GeofenceType::Isochrone)));
    }

    #[test]
    fn test_radar_geofence_to_geofence_result_unknown_type() {
        let mut gf = sample_geofence();
        gf.gf_type = Some("unknown".to_string());
        let result: GeofenceResult = gf.into();

        // Unknown type defaults to Circle
        assert!(matches!(result.geofence_type, Some(GeofenceType::Circle)));
    }

    #[test]
    fn test_radar_geofence_to_geofence_result_no_type() {
        let mut gf = sample_geofence();
        gf.gf_type = None;
        let result: GeofenceResult = gf.into();

        assert!(result.geofence_type.is_none());
    }

    #[test]
    fn test_radar_geofence_empty_fields() {
        let gf = RadarGeofence {
            id: String::new(),
            live: None,
            tag: None,
            external_id: None,
            description: None,
            gf_type: None,
            geometry_center: None,
            metadata: None,
            enabled: None,
            created_at: None,
            geometry: None,
            radius: None,
        };
        let result: GeofenceResult = gf.into();

        assert!(result.id.is_empty());
        assert!(result.tag.is_none());
        assert!(result.external_id.is_none());
        assert!(result.description.is_none());
        assert!(result.geofence_type.is_none());
        assert!(result.geometry_center.is_none());
        assert!(result.metadata.is_none());
        assert!(result.enabled.is_none());
    }
}