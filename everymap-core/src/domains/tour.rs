use crate::domains::routing::TransportMode;
use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Options for tour/sequence optimization.
///
/// Tour optimization involves complex problem definitions (fleet, plan,
/// configuration) that are entirely provider-specific. The core options
/// type is minimal; all provider-specific data goes through `provider_extra`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourOptions {
    /// Transport mode for the tour (car, pedestrian, bicycle, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport_mode: Option<TransportMode>,
    /// Provider-specific problem definition (HERE: TourProblem JSON; other providers: their format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// A stop in an optimized tour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourStop {
    /// The coordinate of this stop
    pub coordinate: Coordinate,
    /// Arrival time (ISO 8601 string, if available)
    pub arrival_time: Option<String>,
    /// Departure time (ISO 8601 string, if available)
    pub departure_time: Option<String>,
    /// Duration at this stop in seconds
    pub duration: Option<f64>,
    /// Distance from previous stop in meters
    pub distance_from_previous: Option<f64>,
}

/// A unified tour response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourResponse {
    /// The optimized sequence of stops
    pub stops: Vec<TourStop>,
    /// Total tour distance in meters
    pub total_distance: Option<f64>,
    /// Total tour duration in seconds
    pub total_duration: Option<f64>,
    /// Number of unassigned stops (that couldn't be fit into the tour)
    pub unassigned_count: Option<u32>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait TourPlanner: Send + Sync {
    async fn optimize_tour(
        &self,
        stops: &[Coordinate],
        options: &TourOptions,
    ) -> EveryMapResult<TourResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tour_options_default() {
        let options = TourOptions::default();
        assert!(options.provider_extra.is_none());
        assert!(options.transport_mode.is_none());
    }

    #[test]
    fn test_tour_options_with_provider_extra() {
        let options = TourOptions {
            transport_mode: None,
            provider_extra: Some(serde_json::json!({"fleet": {"types": []}})),
        };
        assert!(options.provider_extra.is_some());
    }

    #[test]
    fn test_tour_stop_construction() {
        let coordinate = Coordinate::new(52.5, 13.4).unwrap();
        let stop = TourStop {
            coordinate,
            arrival_time: Some("2024-01-01T08:30:00".to_string()),
            departure_time: Some("2024-01-01T09:00:00".to_string()),
            duration: Some(1800.0),
            distance_from_previous: Some(5000.0),
        };
        assert_eq!(stop.arrival_time, Some("2024-01-01T08:30:00".to_string()));
        assert_eq!(stop.duration, Some(1800.0));
    }

    #[test]
    fn test_tour_response_construction() {
        let response = TourResponse {
            stops: vec![],
            total_distance: Some(15000.0),
            total_duration: Some(3600.0),
            unassigned_count: Some(0),
            raw: None,
        };
        assert_eq!(response.total_distance, Some(15000.0));
        assert!(response.stops.is_empty());
    }

    // --- TourResponse serde roundtrip ---

    #[test]
    fn test_tour_response_serde_roundtrip() {
        let response = TourResponse {
            stops: vec![TourStop {
                coordinate: Coordinate::new(52.5, 13.4).unwrap(),
                arrival_time: Some("2024-01-01T08:30:00".to_string()),
                departure_time: Some("2024-01-01T09:00:00".to_string()),
                duration: Some(1800.0),
                distance_from_previous: Some(5000.0),
            }],
            total_distance: Some(15000.0),
            total_duration: Some(3600.0),
            unassigned_count: Some(0),
            raw: Some(serde_json::json!({"optimization": "tsp"})),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TourResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.stops.len(), 1);
        assert_eq!(back.total_distance, Some(15000.0));
        assert_eq!(back.unassigned_count, Some(0));
        assert!(back.raw.is_some());
    }

    #[test]
    fn test_tour_response_empty_serde_roundtrip() {
        let response = TourResponse {
            stops: vec![],
            total_distance: None,
            total_duration: None,
            unassigned_count: None,
            raw: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: TourResponse = serde_json::from_str(&json).unwrap();
        assert!(back.stops.is_empty());
        assert!(back.total_distance.is_none());
    }

    // --- TourStop serde roundtrip ---

    #[test]
    fn test_tour_stop_serde_roundtrip() {
        let stop = TourStop {
            coordinate: Coordinate::new(48.8566, 2.3522).unwrap(),
            arrival_time: Some("2024-06-01T10:00:00".to_string()),
            departure_time: Some("2024-06-01T10:30:00".to_string()),
            duration: Some(1800.0),
            distance_from_previous: Some(2500.0),
        };
        let json = serde_json::to_string(&stop).unwrap();
        let back: TourStop = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coordinate, stop.coordinate);
        assert_eq!(back.arrival_time, stop.arrival_time);
        assert_eq!(back.duration, Some(1800.0));
    }

    // --- TourOptions serde roundtrip ---

    #[test]
    fn test_tour_options_serde_roundtrip() {
        let options = TourOptions {
            transport_mode: None,
            provider_extra: Some(serde_json::json!({
                "fleet": {"types": [{"id": "truck"}]},
                "plan": {"jobs": []}
            })),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: TourOptions = serde_json::from_str(&json).unwrap();
        assert!(back.provider_extra.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_tour_stop_zero_distance() {
        let stop = TourStop {
            coordinate: Coordinate::ORIGIN,
            arrival_time: None,
            departure_time: None,
            duration: Some(0.0),
            distance_from_previous: Some(0.0),
        };
        assert_eq!(stop.distance_from_previous, Some(0.0));
    }

    #[test]
    fn test_tour_response_multiple_stops() {
        let stops: Vec<TourStop> = (0..5)
            .map(|i| TourStop {
                coordinate: Coordinate::new(52.0 + i as f64 * 0.1, 13.0 + i as f64 * 0.1).unwrap(),
                arrival_time: None,
                departure_time: None,
                duration: None,
                distance_from_previous: Some(i as f64 * 1000.0),
            })
            .collect();
        let response = TourResponse {
            stops,
            total_distance: Some(10000.0),
            total_duration: Some(1800.0),
            unassigned_count: None,
            raw: None,
        };
        assert_eq!(response.stops.len(), 5);
    }
}
