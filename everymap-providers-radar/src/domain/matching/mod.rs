pub mod types;

use async_trait::async_trait;
use everymap_core::domains::matching::{MatchingOptions, TraceResponse, MatchedPoint};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use crate::client::RadarClient;
pub use types::*;

const MATCH_BASE_URL: &str = "https://api.radar.io/v1/route/match";

/// Implementation of `RouteMatcher` for Radar Route Match API.
pub struct RadarRouteMatcher {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
}

impl RadarRouteMatcher {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: MATCH_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

impl From<RadarRouteMatchResponse> for TraceResponse {
    fn from(res: RadarRouteMatchResponse) -> Self {
        let raw = serde_json::to_value(&res).unwrap_or_default();

        let matched_points: Vec<MatchedPoint> = res.matched_path.into_iter()
            .map(|p| MatchedPoint {
                coordinate: Coordinate::new(p.latitude, p.longitude).unwrap_or(Coordinate::ORIGIN),
                confidence: None,
                road_name: None,
            })
            .collect();

        let distance = res.distance.as_ref().map(|d| d.value).unwrap_or(0.0);

        Self {
            matched_points,
            distance,
            duration: None,
            raw: Some(raw),
        }
    }
}

#[async_trait]
impl everymap_core::domains::matching::RouteMatcher for RadarRouteMatcher {
    async fn match_route(&self, points: &[Coordinate], _options: &MatchingOptions) -> EveryMapResult<TraceResponse> {
        if points.is_empty() {
            return Err(EveryMapError::ValidationError("At least one point is required for route matching".to_string()));
        }

        let path: Vec<serde_json::Value> = points.iter()
            .map(|p| serde_json::json!({
                "coordinates": format!("{},{}", p.lat, p.lng)
            }))
            .collect();

        let body = serde_json::json!({
            "path": path,
            "mode": "car",
            "geometry": "polyline6"
        });

        let radar_res: RadarRouteMatchResponse = self.client.post_json(&self.base_url, &body).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Route match request failed with status {}", radar_res.meta.code),
            ));
        }

        Ok(TraceResponse::from(radar_res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_route_match_response_to_trace_response() {
        let match_res = RadarRouteMatchResponse {
            meta: crate::domain::types::RadarMeta { code: 200 },
            matched_path: vec![
                crate::domain::matching::types::RadarMatchedPoint {
                    latitude: 40.7128,
                    longitude: -74.006,
                    original_index: Some(0),
                },
                crate::domain::matching::types::RadarMatchedPoint {
                    latitude: 40.713,
                    longitude: -74.005,
                    original_index: Some(1),
                },
            ],
            distance: Some(crate::domain::types::RadarMetric { value: 1200.0, text: "1.2 km".to_string() }),
            geometry: None,
            road_attributes: None,
        };
        let result: TraceResponse = match_res.into();

        assert_eq!(result.matched_points.len(), 2);
        assert!((result.distance - 1200.0).abs() < f64::EPSILON);
        assert!(result.duration.is_none());
        assert!((result.matched_points[0].coordinate.lat - 40.7128).abs() < f64::EPSILON);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_radar_route_match_response_no_distance() {
        let match_res = RadarRouteMatchResponse {
            meta: crate::domain::types::RadarMeta { code: 200 },
            matched_path: vec![],
            distance: None,
            geometry: None,
            road_attributes: None,
        };
        let result: TraceResponse = match_res.into();

        assert!(result.matched_points.is_empty());
        assert!((result.distance - 0.0).abs() < f64::EPSILON);
    }
}