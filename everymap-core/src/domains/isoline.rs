use async_trait::async_trait;
use crate::types::Coordinate;
use crate::domains::routing::TransportMode;
use crate::error::EveryMapResult;
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
    /// Departure time (ISO 8601 string)
    pub departure_time: Option<String>,
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
    async fn get_isoline(&self, center: &Coordinate, range: f64, options: &IsolineOptions) -> EveryMapResult<IsolineResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isoline_options_default() {
        let opts = IsolineOptions::default();
        assert!(opts.range_type.is_none());
        assert!(opts.transport_mode.is_none());
        assert!(opts.departure_time.is_none());
        assert!(opts.avoid.is_empty());
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_isoline_options_with_fields() {
        let opts = IsolineOptions {
            range_type: Some(RangeType::Time),
            transport_mode: Some(crate::domains::routing::TransportMode::Car),
            departure_time: Some("2024-01-01T08:00:00".to_string()),
            avoid: vec![crate::domains::routing::AvoidType::Tolls],
            provider_extra: Some(serde_json::json!({"routing_mode": "fast"})),
        };
        assert_eq!(opts.range_type, Some(RangeType::Time));
        assert!(opts.avoid.len() == 1);
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
}