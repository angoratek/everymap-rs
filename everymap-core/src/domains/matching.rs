use crate::domains::routing::{DepartureTime, TransportMode};
use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for GPS trace matching to the road network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchingOptions {
    /// Transport mode for matching
    pub transport_mode: Option<TransportMode>,
    /// Heading angle in degrees (0-360)
    pub heading: Option<f64>,
    /// Departure time
    pub departure_time: Option<DepartureTime>,
    /// Route restrictions
    pub avoid: Vec<crate::domains::routing::AvoidType>,
    /// Provider-specific options (HERE: map_match_radius, route_match, vehicle params; Google: interpolation, snapping)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// A matched point from route matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedPoint {
    /// The matched/snapped coordinate on the road network
    pub coordinate: Coordinate,
    /// Confidence score for this match (0.0-1.0)
    pub confidence: Option<f64>,
    /// Matched road name (if available)
    pub road_name: Option<String>,
}

/// A unified matching response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResponse {
    /// The snapped/matched points
    pub matched_points: Vec<MatchedPoint>,
    /// Total matched route distance in meters
    pub distance: f64,
    /// Total matched route duration in seconds
    pub duration: Option<f64>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait RouteMatcher: Send + Sync {
    async fn match_route(
        &self,
        points: &[Coordinate],
        options: &MatchingOptions,
    ) -> EveryMapResult<TraceResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_options_default() {
        let opts = MatchingOptions::default();
        assert!(opts.transport_mode.is_none());
        assert!(opts.heading.is_none());
        assert!(opts.departure_time.is_none());
        assert!(opts.avoid.is_empty());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_matching_options_with_fields() {
        let opts = MatchingOptions {
            transport_mode: Some(crate::domains::routing::TransportMode::Car),
            heading: Some(180.0),
            departure_time: Some(DepartureTime::Iso8601("2024-01-01T08:00:00".to_string())),
            avoid: vec![crate::domains::routing::AvoidType::Tolls],
            provider_extra: Some(serde_json::json!({"map_match_radius": 50})),
        };
        assert_eq!(opts.heading, Some(180.0));
        assert!(opts.avoid.len() == 1);
    }

    #[test]
    fn test_matched_point_construction() {
        let coord = Coordinate::new(52.5, 13.4).unwrap();
        let point = MatchedPoint {
            coordinate: coord,
            confidence: Some(0.95),
            road_name: Some("Main Street".to_string()),
        };
        assert_eq!(point.confidence, Some(0.95));
        assert_eq!(point.road_name, Some("Main Street".to_string()));
    }

    #[test]
    fn test_trace_response_construction() {
        let response = TraceResponse {
            matched_points: vec![],
            distance: 5000.0,
            duration: Some(600.0),
            raw: None,
        };
        assert_eq!(response.distance, 5000.0);
        assert!(response.matched_points.is_empty());
    }

    // --- TraceResponse serde roundtrip ---

    #[test]
    fn test_trace_response_serde_roundtrip() {
        let response = TraceResponse {
            matched_points: vec![MatchedPoint {
                coordinate: Coordinate::new(52.5, 13.4).unwrap(),
                confidence: Some(0.95),
                road_name: Some("Friedrichstr".to_string()),
            }],
            distance: 12000.5,
            duration: Some(900.0),
            raw: Some(serde_json::json!({"trace_id": "t1"})),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TraceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.matched_points.len(), 1);
        assert_eq!(back.distance, 12000.5);
        assert_eq!(back.duration, Some(900.0));
        assert!(back.raw.is_some());
    }

    #[test]
    fn test_trace_response_empty_serde_roundtrip() {
        let response = TraceResponse {
            matched_points: vec![],
            distance: 0.0,
            duration: None,
            raw: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TraceResponse = serde_json::from_str(&json).unwrap();
        assert!(back.matched_points.is_empty());
        assert_eq!(back.distance, 0.0);
        assert!(back.duration.is_none());
    }

    // --- MatchingOptions serde roundtrip ---

    #[test]
    fn test_matching_options_serde_roundtrip() {
        let opts = MatchingOptions {
            transport_mode: Some(crate::domains::routing::TransportMode::Bicycle),
            heading: Some(270.0),
            departure_time: Some(DepartureTime::Iso8601("2024-03-15T10:00:00".to_string())),
            avoid: vec![crate::domains::routing::AvoidType::Highways],
            provider_extra: Some(serde_json::json!({"map_match_radius": 30})),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: MatchingOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.transport_mode,
            Some(crate::domains::routing::TransportMode::Bicycle)
        );
        assert_eq!(back.heading, Some(270.0));
        assert_eq!(back.avoid.len(), 1);
        assert!(back.provider_extra.is_some());
    }

    // --- MatchedPoint serde roundtrip ---

    #[test]
    fn test_matched_point_serde_roundtrip() {
        let point = MatchedPoint {
            coordinate: Coordinate::new(48.8566, 2.3522).unwrap(),
            confidence: Some(0.88),
            road_name: Some("Champs-Elysees".to_string()),
        };
        let json = serde_json::to_string(&point).unwrap();
        let back: MatchedPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coordinate, point.coordinate);
        assert_eq!(back.confidence, point.confidence);
        assert_eq!(back.road_name, point.road_name);
    }

    // --- Edge cases ---

    #[test]
    fn test_matched_point_zero_confidence() {
        let point = MatchedPoint {
            coordinate: Coordinate::ORIGIN,
            confidence: Some(0.0),
            road_name: None,
        };
        assert_eq!(point.confidence, Some(0.0));
        assert!(point.road_name.is_none());
    }

    #[test]
    fn test_trace_response_zero_distance() {
        let response = TraceResponse {
            matched_points: vec![],
            distance: 0.0,
            duration: Some(0.0),
            raw: None,
        };
        assert_eq!(response.distance, 0.0);
        assert_eq!(response.duration, Some(0.0));
    }
}
