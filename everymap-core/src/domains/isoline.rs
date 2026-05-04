use crate::domains::routing::{DepartureTime, TransportMode};
use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Type of range for isoline calculation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RangeType {
    Distance,
    Time,
    Consumption,
}

/// Options for isoline (reachability polygon) calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsolineOptions {
    /// Type of range (distance, time, or consumption)
    pub range_type: Option<RangeType>,
    /// Transport mode for the isoline calculation
    pub transport_mode: Option<TransportMode>,
    /// Departure time
    pub departure_time: Option<DepartureTime>,
    /// Route restrictions
    pub avoid: Vec<crate::domains::routing::AvoidType>,
    /// Provider-specific options (HERE: routing_mode, optimize_for, vehicle params)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// A unified isoline result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineResult {
    /// The polygon points defining the isoline boundary
    pub polygon: Vec<Coordinate>,
    /// The range value (in meters or seconds, depending on range type)
    pub range: Option<f64>,
}

/// A unified isoline response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineResponse {
    /// The isoline results (may contain multiple ranges)
    pub isolines: Vec<IsolineResult>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait IsolineProvider: Send + Sync {
    async fn get_isoline(
        &self,
        center: &Coordinate,
        range: f64,
        options: &IsolineOptions,
    ) -> EveryMapResult<IsolineResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isoline_options_default() {
        let options = IsolineOptions::default();
        assert!(options.range_type.is_none());
        assert!(options.transport_mode.is_none());
        assert!(options.departure_time.is_none());
        assert!(options.avoid.is_empty());
        assert!(options.provider_extra.is_none());
    }

    #[test]
    fn test_isoline_options_with_fields() {
        let options = IsolineOptions {
            range_type: Some(RangeType::Time),
            transport_mode: Some(crate::domains::routing::TransportMode::Car),
            departure_time: Some(DepartureTime::Iso8601("2024-01-01T08:00:00".to_string())),
            avoid: vec![crate::domains::routing::AvoidType::Tolls],
            provider_extra: Some(serde_json::json!({"routing_mode": "fast"})),
        };
        assert_eq!(options.range_type, Some(RangeType::Time));
        assert!(options.avoid.len() == 1);
    }

    #[test]
    fn test_isoline_result_construction() {
        let result = IsolineResult {
            polygon: vec![],
            range: Some(1000.0),
        };
        assert_eq!(result.range, Some(1000.0));
        assert!(result.polygon.is_empty());
    }

    #[test]
    fn test_range_type_serialization() {
        assert_eq!(
            serde_json::to_string(&RangeType::Distance).unwrap(),
            "\"Distance\""
        );
        let rt: RangeType = serde_json::from_str("\"Time\"").unwrap();
        assert_eq!(rt, RangeType::Time);
    }

    // --- RangeType all 3 variants serde roundtrip ---

    #[test]
    fn test_range_type_all_variants_serde() {
        let variants = [RangeType::Distance, RangeType::Time, RangeType::Consumption];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: RangeType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "Failed roundtrip for {:?}", v);
        }
    }

    #[test]
    fn test_range_type_all_variants_distinct() {
        let variants = [RangeType::Distance, RangeType::Time, RangeType::Consumption];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i != j {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    // --- IsolineResponse serde roundtrip ---

    #[test]
    fn test_isoline_response_serde_roundtrip() {
        let response = IsolineResponse {
            isolines: vec![IsolineResult {
                polygon: vec![
                    Coordinate::new(52.5, 13.4).unwrap(),
                    Coordinate::new(52.6, 13.5).unwrap(),
                ],
                range: Some(5000.0),
            }],
            raw: Some(serde_json::json!({"center": "52.5,13.4"})),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: IsolineResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.isolines.len(), 1);
        assert_eq!(back.isolines[0].polygon.len(), 2);
        assert_eq!(back.isolines[0].range, Some(5000.0));
        assert!(back.raw.is_some());
    }

    #[test]
    fn test_isoline_response_empty() {
        let response = IsolineResponse {
            isolines: vec![],
            raw: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: IsolineResponse = serde_json::from_str(&json).unwrap();
        assert!(back.isolines.is_empty());
        assert!(back.raw.is_none());
    }

    // --- IsolineOptions serde roundtrip ---

    #[test]
    fn test_isoline_options_serde_roundtrip() {
        let options = IsolineOptions {
            range_type: Some(RangeType::Consumption),
            transport_mode: Some(crate::domains::routing::TransportMode::Truck),
            departure_time: Some(DepartureTime::Iso8601("2024-01-01T08:00:00".to_string())),
            avoid: vec![
                crate::domains::routing::AvoidType::Tolls,
                crate::domains::routing::AvoidType::Highways,
            ],
            provider_extra: Some(serde_json::json!({"optimize_for": "quality"})),
        };
        let json = serde_json::to_string(&options).unwrap();
        let back: IsolineOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.range_type, Some(RangeType::Consumption));
        assert_eq!(back.avoid.len(), 2);
        assert!(back.provider_extra.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_isoline_result_empty_polygon() {
        let result = IsolineResult {
            polygon: vec![],
            range: None,
        };
        assert!(result.polygon.is_empty());
        assert!(result.range.is_none());
    }

    #[test]
    fn test_isoline_result_zero_range() {
        let result = IsolineResult {
            polygon: vec![Coordinate::ORIGIN],
            range: Some(0.0),
        };
        assert_eq!(result.range, Some(0.0));
    }
}
