pub mod types;

use crate::client::RadarClient;
use crate::endpoints::{endpoint, MATCH_ROUTE_PATH, RADAR_API_ROOT};
use crate::error_mapping::{ensure_success, invalid_response};
use async_trait::async_trait;
use everymap_core::domains::matching::{MatchedPoint, MatchingOptions, TraceResponse};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
pub use types::*;

/// Implementation of `RouteMatcher` for Radar Route Match API.
pub struct RadarRouteMatcher {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) service_root: String,
}

impl RadarRouteMatcher {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            service_root: RADAR_API_ROOT.to_string(),
        }
    }

    /// Creates a matcher against a Radar service root.
    ///
    /// # Parameters
    /// - `client` — the shared Radar client.
    /// - `service_root` — the scheme and host serving the API, for example
    ///   `https://api.radar.io` or a test server's origin. Every Radar
    ///   endpoint is derived from this one root.
    pub fn with_base_url(client: std::sync::Arc<RadarClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The route matching endpoint.
    pub(crate) fn endpoint(&self) -> String {
        endpoint(&self.service_root, MATCH_ROUTE_PATH)
    }
}

impl TryFrom<RadarRouteMatchResponse> for TraceResponse {
    type Error = EveryMapError;

    fn try_from(response: RadarRouteMatchResponse) -> Result<Self, Self::Error> {
        let raw = serde_json::to_value(&response).ok();

        let mut matched_points = Vec::with_capacity(response.matched_path.len());
        for point in response.matched_path {
            let latitude = point
                .latitude
                .ok_or_else(|| invalid_response("matched point lacks a latitude"))?;
            let longitude = point
                .longitude
                .ok_or_else(|| invalid_response("matched point lacks a longitude"))?;
            let coordinate = Coordinate::new(latitude, longitude)
                .map_err(|_| invalid_response("matched point carries an invalid location"))?;
            matched_points.push(MatchedPoint {
                coordinate,
                confidence: None,
                road_name: None,
            });
        }

        let distance = response
            .distance
            .as_ref()
            .and_then(|metric| metric.value)
            .ok_or_else(|| invalid_response("route match lacks a distance"))?;

        Ok(Self {
            matched_points,
            distance,
            duration: None,
            raw,
        })
    }
}

#[async_trait]
impl everymap_core::domains::matching::RouteMatcher for RadarRouteMatcher {
    async fn match_route(
        &self,
        points: &[Coordinate],
        options: &MatchingOptions,
    ) -> EveryMapResult<TraceResponse> {
        if options.heading.is_some() {
            log::warn!("Radar Route Match API does not support heading; ignoring");
        }
        if options.departure_time.is_some() {
            log::warn!("Radar Route Match API does not support departure_time; ignoring");
        }
        if !options.avoid.is_empty() {
            log::warn!("Radar Route Match API does not support avoid restrictions; ignoring");
        }

        if points.is_empty() {
            return Err(EveryMapError::ValidationError(
                "At least one point is required for route matching".to_string(),
            ));
        }

        let path: Vec<serde_json::Value> = points
            .iter()
            .map(|p| {
                serde_json::json!({
                    "coordinates": format!("{},{}", p.lat, p.lng)
                })
            })
            .collect();

        let mode = options
            .transport_mode
            .as_ref()
            .map(crate::domain::transport::radar_transport_mode)
            .unwrap_or("car");

        let body = serde_json::json!({
            "path": path,
            "mode": mode,
            "geometry": "polyline6"
        });

        let url = self.endpoint();
        let radar_res: RadarRouteMatchResponse = self.client.post_json(&url, &body).await?;
        ensure_success(radar_res.meta.code)?;

        TraceResponse::try_from(radar_res)
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
                    latitude: Some(40.7128),
                    longitude: Some(-74.006),
                    original_index: Some(0),
                },
                crate::domain::matching::types::RadarMatchedPoint {
                    latitude: Some(40.713),
                    longitude: Some(-74.005),
                    original_index: Some(1),
                },
            ],
            distance: Some(crate::domain::types::RadarMetric {
                value: Some(1200.0),
                text: "1.2 km".to_string(),
            }),
            geometry: None,
            road_attributes: None,
        };
        let result = TraceResponse::try_from(match_res).expect("the fixture match converts");

        assert_eq!(result.matched_points.len(), 2);
        assert!((result.distance - 1200.0).abs() < f64::EPSILON);
        assert!(result.duration.is_none());
        assert!((result.matched_points[0].coordinate.lat - 40.7128).abs() < f64::EPSILON);
        assert!(result.raw.is_some());
    }

    #[test]
    fn a_match_without_a_distance_is_invalid() {
        let match_res = RadarRouteMatchResponse {
            meta: crate::domain::types::RadarMeta { code: 200 },
            matched_path: vec![],
            distance: None,
            geometry: None,
            road_attributes: None,
        };
        match TraceResponse::try_from(match_res) {
            Err(EveryMapError::ProviderError { code, .. }) => {
                assert_eq!(code, crate::error_mapping::INVALID_RESPONSE_CODE);
            }
            other => panic!("expected an invalid-response error, got {other:?}"),
        }
    }
}
