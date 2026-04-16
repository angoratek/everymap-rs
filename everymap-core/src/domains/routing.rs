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

    // --- AvoidType all 5 variants serde roundtrip ---

    #[test]
    fn test_avoid_type_all_variants_serde() {
        let variants = [
            AvoidType::Tolls,
            AvoidType::Ferries,
            AvoidType::Tunnels,
            AvoidType::Highways,
            AvoidType::DirtRoads,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: AvoidType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_avoid_type_all_variants_distinct() {
        let variants = [
            AvoidType::Tolls,
            AvoidType::Ferries,
            AvoidType::Tunnels,
            AvoidType::Highways,
            AvoidType::DirtRoads,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- TransportMode all 8 variants serde roundtrip ---

    #[test]
    fn test_transport_mode_all_variants_serde() {
        let variants = [
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
            TransportMode::Unknown,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: TransportMode = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_transport_mode_all_variants_distinct() {
        let variants = [
            TransportMode::Car,
            TransportMode::Truck,
            TransportMode::Pedestrian,
            TransportMode::Bicycle,
            TransportMode::Scooter,
            TransportMode::Bus,
            TransportMode::Taxi,
            TransportMode::Unknown,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- RouteResponse serde roundtrip ---

    #[test]
    fn test_route_response_serde_roundtrip() {
        let response = RouteResponse {
            routes: vec![RouteResult {
                distance: 15000.0,
                duration: 1800.0,
                geometry: Polyline::new(vec![]),
                transport_mode: Some(TransportMode::Car),
                steps: vec![],
                bounding_box: None,
                raw: None,
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: RouteResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.routes.len(), 1);
        assert_eq!(back.routes[0].distance, 15000.0);
        assert_eq!(back.routes[0].transport_mode, Some(TransportMode::Car));
    }

    #[test]
    fn test_route_response_empty_routes() {
        let response = RouteResponse { routes: vec![] };
        let json = serde_json::to_string(&response).unwrap();
        let back: RouteResponse = serde_json::from_str(&json).unwrap();
        assert!(back.routes.is_empty());
    }

    // --- RouteOptions with provider_extra serde roundtrip ---

    #[test]
    fn test_route_options_serde_roundtrip() {
        let opts = RouteOptions {
            transport_mode: Some(TransportMode::Truck),
            alternatives: Some(2),
            avoid: vec![AvoidType::Tolls, AvoidType::Ferries],
            departure_time: Some("2024-06-01T08:00:00".to_string()),
            arrival_time: None,
            language: Some("en".to_string()),
            provider_extra: Some(serde_json::json!({"routing_mode": "fast", "truck": {"weight": 18}})),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: RouteOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transport_mode, Some(TransportMode::Truck));
        assert_eq!(back.alternatives, Some(2));
        assert_eq!(back.avoid.len(), 2);
        assert!(back.provider_extra.is_some());
    }

    // --- RouteStep serde roundtrip ---

    #[test]
    fn test_route_step_serde_roundtrip() {
        let step = RouteStep {
            instruction: Some("Turn right onto Main St".to_string()),
            distance: Some(500.0),
            duration: Some(60.0),
            start_coordinate: Some(Coordinate::new(52.5, 13.4).unwrap()),
            end_coordinate: Some(Coordinate::new(52.51, 13.41).unwrap()),
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: RouteStep = serde_json::from_str(&json).unwrap();
        assert_eq!(back.instruction.as_deref(), Some("Turn right onto Main St"));
        assert_eq!(back.distance, Some(500.0));
        assert_eq!(back.duration, Some(60.0));
    }

    // --- RouteResult with all fields populated ---

    #[test]
    fn test_route_result_full_serde_roundtrip() {
        let result = RouteResult {
            distance: 0.0,
            duration: 0.0,
            geometry: Polyline::new(vec![Coordinate::new(52.5, 13.4).unwrap()]),
            transport_mode: Some(TransportMode::Pedestrian),
            steps: vec![RouteStep {
                instruction: Some("Walk north".to_string()),
                distance: Some(100.0),
                duration: Some(120.0),
                start_coordinate: None,
                end_coordinate: None,
            }],
            bounding_box: Some(BoundingBox::new(
                Coordinate::new(52.6, 13.5).unwrap(),
                Coordinate::new(52.4, 13.3).unwrap(),
            )),
            raw: Some(serde_json::json!({"legs": []})),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: RouteResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.distance, 0.0);
        assert_eq!(back.steps.len(), 1);
        assert!(back.bounding_box.is_some());
        assert!(back.raw.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_route_result_zero_distance_duration() {
        let result = RouteResult {
            distance: 0.0,
            duration: 0.0,
            geometry: Polyline::new(vec![]),
            transport_mode: None,
            steps: vec![],
            bounding_box: None,
            raw: None,
        };
        assert_eq!(result.distance, 0.0);
        assert_eq!(result.duration, 0.0);
    }

    #[test]
    fn test_route_options_all_avoid_types() {
        let opts = RouteOptions {
            avoid: vec![
                AvoidType::Tolls,
                AvoidType::Ferries,
                AvoidType::Tunnels,
                AvoidType::Highways,
                AvoidType::DirtRoads,
            ],
            ..Default::default()
        };
        assert_eq!(opts.avoid.len(), 5);
    }
}