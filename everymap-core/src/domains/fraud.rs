use async_trait::async_trait;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for a fraud check via a track/position update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FraudCheckOptions {
    /// Device ID submitting the location
    pub device_id: String,
    /// Latitude of the reported position
    pub latitude: f64,
    /// Longitude of the reported position
    pub longitude: f64,
    /// Accuracy of the reported position in meters
    pub accuracy: f64,
    /// Stable user ID (identifies logged-in users across devices)
    pub user_id: Option<String>,
    /// Whether the client is in the foreground
    pub foreground: Option<bool>,
    /// Whether the user is stopped vs. moving
    pub stopped: Option<bool>,
    /// Arbitrary metadata key-value pairs
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Device type (e.g., "iOS", "Android", "Web")
    pub device_type: Option<String>,
    /// Provider-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
}

/// A unified fraud detection result from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudResult {
    /// Whether the location is verified as legitimate
    pub verified: bool,
    /// Whether the fraud checks passed
    pub passed: bool,
    /// Whether GPS mocking was detected
    pub mocked: bool,
    /// Whether a location jump was detected (impossible travel)
    pub jumped: bool,
    /// Whether a compromised device was detected
    pub compromised: bool,
    /// Whether the reported accuracy is suspiciously low
    pub inaccurate: bool,
    /// Whether a proxy or VPN was detected
    pub proxy: bool,
    /// Whether location sharing spoofing was detected
    pub sharing: bool,
    /// Whether the user was blocked due to fraud
    pub blocked: bool,
    /// Whether fraud checks were bypassed
    pub bypassed: bool,
    /// Provider-specific raw data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// Core trait for fraud detection providers.
///
/// Detects GPS spoofing, proxy/VPN usage, device tampering, and location
/// sharing spoofing via a track/position update. Radar is the primary
/// provider; other providers return `UnsupportedDomain`.
#[async_trait]
pub trait FraudDetector: Send + Sync {
    /// Submit a location update and get fraud detection results.
    async fn check_fraud(&self, options: &FraudCheckOptions) -> EveryMapResult<FraudResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraud_check_options_required_fields() {
        let opts = FraudCheckOptions {
            device_id: "dev123".to_string(),
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy: 10.0,
            ..Default::default()
        };
        assert_eq!(opts.device_id, "dev123");
        assert!((opts.latitude - 40.7128).abs() < f64::EPSILON);
        assert!((opts.longitude - (-74.0060)).abs() < f64::EPSILON);
        assert!((opts.accuracy - 10.0).abs() < f64::EPSILON);
        assert!(opts.user_id.is_none());
        assert!(opts.foreground.is_none());
        assert!(opts.metadata.is_none());
    }

    #[test]
    fn test_fraud_result_default_like() {
        let result = FraudResult {
            verified: true,
            passed: true,
            mocked: false,
            jumped: false,
            compromised: false,
            inaccurate: false,
            proxy: false,
            sharing: false,
            blocked: false,
            bypassed: false,
            raw: None,
        };
        assert!(result.verified);
        assert!(result.passed);
        assert!(!result.mocked);
        assert!(result.raw.is_none());
    }

    #[test]
    fn test_fraud_result_serde_roundtrip() {
        let result = FraudResult {
            verified: true,
            passed: false,
            mocked: true,
            jumped: true,
            compromised: false,
            inaccurate: false,
            proxy: true,
            sharing: false,
            blocked: false,
            bypassed: false,
            raw: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: FraudResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.verified, back.verified);
        assert_eq!(result.passed, back.passed);
        assert_eq!(result.mocked, back.mocked);
        assert_eq!(result.jumped, back.jumped);
        assert_eq!(result.proxy, back.proxy);
    }

    #[test]
    fn test_fraud_check_options_with_all_optional_fields() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("session_id".to_string(), serde_json::json!("abc123"));
        let opts = FraudCheckOptions {
            device_id: "dev_1".to_string(),
            latitude: 40.7128,
            longitude: -74.006,
            accuracy: 10.0,
            user_id: Some("user_1".to_string()),
            foreground: Some(true),
            stopped: Some(false),
            metadata: Some(metadata),
            device_type: Some("iOS".to_string()),
            provider_extra: None,
        };
        assert_eq!(opts.device_id, "dev_1");
        assert!((opts.latitude - 40.7128).abs() < f64::EPSILON);
        assert!((opts.longitude - (-74.006)).abs() < f64::EPSILON);
        assert!((opts.accuracy - 10.0).abs() < f64::EPSILON);
        assert_eq!(opts.user_id.as_deref(), Some("user_1"));
        assert_eq!(opts.foreground, Some(true));
        assert_eq!(opts.stopped, Some(false));
        assert_eq!(opts.device_type.as_deref(), Some("iOS"));
        assert!(opts.metadata.unwrap().contains_key("session_id"));
    }

    #[test]
    fn test_fraud_result_with_raw() {
        let raw = serde_json::json!({"extra": "data"});
        let result = FraudResult {
            verified: false,
            passed: false,
            mocked: false,
            jumped: false,
            compromised: false,
            inaccurate: false,
            proxy: false,
            sharing: false,
            blocked: true,
            bypassed: false,
            raw: Some(raw.clone()),
        };
        assert!(result.blocked);
        assert!(result.raw.is_some());
        assert_eq!(result.raw.unwrap()["extra"], "data");
    }
}