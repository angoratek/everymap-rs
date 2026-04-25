use crate::error::EveryMapResult;
use crate::types::Coordinate;
use async_trait::async_trait;
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
    async fn get_position(
        &self,
        options: &PositioningOptions,
    ) -> EveryMapResult<PositioningResponse>;
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

    // --- PositioningResponse serde roundtrip ---

    #[test]
    fn test_positioning_response_serde_roundtrip() {
        let coord = Coordinate::new(52.5, 13.4).unwrap();
        let response = PositioningResponse {
            coordinate: coord,
            accuracy: Some(50.0),
            altitude: Some(34.0),
            altitude_accuracy: Some(10.0),
            raw: Some(serde_json::json!({"wlan_count": 5})),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: PositioningResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coordinate.lat, 52.5);
        assert_eq!(back.accuracy, Some(50.0));
        assert_eq!(back.altitude, Some(34.0));
        assert!(back.raw.is_some());
    }

    #[test]
    fn test_positioning_response_minimal_serde_roundtrip() {
        let coord = Coordinate::ORIGIN;
        let response = PositioningResponse {
            coordinate: coord,
            accuracy: None,
            altitude: None,
            altitude_accuracy: None,
            raw: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: PositioningResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coordinate, coord);
        assert!(back.accuracy.is_none());
    }

    // --- PositioningOptions serde roundtrip ---

    #[test]
    fn test_positioning_options_serde_roundtrip() {
        let opts = PositioningOptions {
            provider_extra: Some(serde_json::json!({
                "wlan": [{"mac": "aa:bb:cc:dd:ee:ff", "signalStrength": -70}],
                "cell": [{"mcc": 262, "mnc": 1, "cid": 12345}]
            })),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: PositioningOptions = serde_json::from_str(&json).unwrap();
        assert!(back.provider_extra.is_some());
    }

    // --- Edge cases ---

    #[test]
    fn test_positioning_response_boundary_coordinates() {
        let coord = Coordinate::new(90.0, 180.0).unwrap();
        let response = PositioningResponse {
            coordinate: coord,
            accuracy: Some(0.0),
            altitude: Some(0.0),
            altitude_accuracy: Some(0.0),
            raw: None,
        };
        assert_eq!(response.coordinate.lat, 90.0);
        assert_eq!(response.accuracy, Some(0.0));
    }

    #[test]
    fn test_positioning_response_negative_altitude() {
        let coord = Coordinate::new(-33.8688, 151.2093).unwrap();
        let response = PositioningResponse {
            coordinate: coord,
            accuracy: Some(15.0),
            altitude: Some(-11.0),
            altitude_accuracy: Some(5.0),
            raw: None,
        };
        assert_eq!(response.altitude, Some(-11.0));
    }
}
