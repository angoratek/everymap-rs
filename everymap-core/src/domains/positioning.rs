use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Options for network-based positioning.
///
/// Network positioning uses WiFi, cell tower, and Bluetooth observations
/// to estimate a device's location. The observation data is provider-specific
/// and passed via `provider_extra`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositioningOptions {
    /// Provider-specific observation data (HERE: wlan/cell/bluetooth arrays; Google: considerIp, wifiAccessPoints, cellTowers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// A unified positioning response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositioningResponse {
    /// The estimated position
    pub coordinate: Coordinate,
    /// Accuracy of the position estimate in meters
    pub accuracy: Option<f64>,
    /// Altitude in meters above sea level (if available)
    pub altitude: Option<f64>,
    /// Accuracy of the altitude estimate in meters
    pub altitude_accuracy: Option<f64>,
    /// Provider-specific raw data for advanced use cases
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[async_trait]
pub trait NetworkPositioner: Send + Sync {
    async fn get_position(&self, options: &PositioningOptions) -> EveryMapResult<PositioningResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positioning_options_default() {
        let opts = PositioningOptions::default();
        assert!(opts.provider_extra.is_none());
    }

    #[test]
    fn test_positioning_options_with_provider_extra() {
        let opts = PositioningOptions {
            provider_extra: Some(serde_json::json!({"wlan": [{"mac": "aa:bb:cc:dd:ee:ff"}]})),
        };
        assert!(opts.provider_extra.is_some());
    }

    #[test]
    fn test_positioning_response_construction() {
        let coord = Coordinate::new(52.5, 13.4).unwrap();
        let response = PositioningResponse {
            coordinate: coord,
            accuracy: Some(50.0),
            altitude: Some(34.0),
            altitude_accuracy: Some(10.0),
            raw: None,
        };
        assert_eq!(response.coordinate.lat, 52.5);
        assert_eq!(response.accuracy, Some(50.0));
        assert_eq!(response.altitude, Some(34.0));
    }
}