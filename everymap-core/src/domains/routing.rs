use async_trait::async_trait;
use crate::types::{Coordinate, Polyline, BoundingBox};
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Avoid types for routing restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AvoidType {
    Tolls,
    Ferries,
    Tunnels,
    Highways,
    DirtRoads,
}

/// Options for route calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteOptions {
    /// Transport mode for the route
    pub transport_mode: Option<TransportMode>,
    /// Number of alternative routes to compute
    pub alternatives: Option<u32>,
    /// Route restrictions (tolls, ferries, highways, etc.)
    pub avoid: Vec<AvoidType>,
    /// Departure time (ISO 8601 string)
    pub departure_time: Option<String>,
    /// Arrival time (ISO 8601 string)
    pub arrival_time: Option<String>,
    /// Preferred response language (BCP 47 language tag)
    pub language: Option<String>,
    /// Provider-specific options (HERE: routing_mode, spans, truck params; Google: waypoints, traffic_model)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// Transport mode for a route.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransportMode {
    Car,
    Truck,
    Pedestrian,
    Bicycle,
    Scooter,
    Bus,
    Taxi,
    Unknown,
}

/// A single leg/step in a route (e.g., a turn-by-turn instruction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    /// Instruction text (e.g., "Turn right onto Main St")
    pub instruction: Option<String>,
    /// Distance of this step in meters
    pub distance: Option<f64>,
    /// Duration of this step in seconds
    pub duration: Option<f64>,
    /// Starting coordinate of this step
    pub start_coordinate: Option<Coordinate>,
    /// Ending coordinate of this step
    pub end_coordinate: Option<Coordinate>,
}

/// A unified route result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    /// Total route distance in meters
    pub distance: f64,
    /// Total route duration in seconds
    pub duration: f64,
    /// Route geometry as a polyline
    pub geometry: Polyline,
    /// Transport mode used for this route
    pub transport_mode: Option<TransportMode>,
    /// Turn-by-turn steps (if available)
    pub steps: Vec<RouteStep>,
    /// Bounding box for the route (if available)
    pub bounding_box: Option<BoundingBox>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// A unified route response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    /// The route results (may contain alternatives)
    pub routes: Vec<RouteResult>,
}

#[async_trait]
pub trait Router: Send + Sync {
    async fn calculate_route(&self, start: &Coordinate, end: &Coordinate, options: &RouteOptions) -> EveryMapResult<RouteResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_options_default() {
        let opts = RouteOptions::default();
        assert!(opts.transport_mode.is_none());
        assert!(opts.alternatives.is_none());
        assert!(opts.avoid.is_empty());
        assert!(opts.departure_time.is_none());
        assert!(opts.arrival_time.is_none());
        assert!(opts.language.is_none());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_route_options_with_fields() {
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Car),
            alternatives: Some(3),
            avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
            language: Some("de-DE".to_string()),
            provider_extra: Some(serde_json::json!({"routing_mode": "fast"})),
            ..Default::default()
        };
        assert_eq!(opts.transport_mode, Some(TransportMode::Car));
        assert_eq!(opts.alternatives, Some(3));
        assert_eq!(opts.avoid.len(), 2);
    }

    #[test]
    fn test_transport_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&TransportMode::Car).unwrap(),
            "\"Car\""
        );
        let mode: TransportMode = serde_json::from_str("\"Truck\"").unwrap();
        assert_eq!(mode, TransportMode::Truck);
    }

    #[test]
    fn test_avoid_type_serialization() {
        assert_eq!(
            serde_json::to_string(&AvoidType::Tolls).unwrap(),
            "\"Tolls\""
        );
        let avoid: AvoidType = serde_json::from_str("\"Highways\"").unwrap();
        assert_eq!(avoid, AvoidType::Highways);
    }

    #[test]
    fn test_route_result_construction() {
        let result = RouteResult {
            distance: 15000.0,
            duration: 1800.0,
            geometry: Polyline::new(vec![]),
            transport_mode: Some(TransportMode::Car),
            steps: vec![],
            bounding_box: None,
            raw: None,
        };
        assert_eq!(result.distance, 15000.0);
        assert_eq!(result.duration, 1800.0);
        assert_eq!(result.transport_mode, Some(TransportMode::Car));
    }
}