pub mod types;

use async_trait::async_trait;
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions, FraudResult};
use everymap_core::error::{EveryMapError, EveryMapResult};
use crate::client::RadarClient;
pub use types::*;

const TRACK_URL: &str = "https://api.radar.io/v1/track";

/// Implementation of `FraudDetector` for Radar.
///
/// Uses Radar's Track API which returns fraud detection fields in the
/// user object of the response.
pub struct RadarFraudDetector {
    pub(crate) client: std::sync::Arc<RadarClient>,
    pub(crate) base_url: String,
}

impl RadarFraudDetector {
    pub fn new(client: std::sync::Arc<RadarClient>) -> Self {
        Self {
            client,
            base_url: TRACK_URL.to_string(),
        }
    }

    pub fn with_base_url(client: std::sync::Arc<RadarClient>, base_url: String) -> Self {
        Self { client, base_url }
    }
}

impl From<RadarFraudData> for FraudResult {
    fn from(fraud: RadarFraudData) -> Self {
        Self {
            verified: fraud.verified,
            passed: fraud.passed,
            mocked: fraud.mocked,
            jumped: fraud.jumped,
            compromised: fraud.compromised,
            inaccurate: fraud.inaccurate,
            proxy: fraud.proxy,
            sharing: fraud.sharing,
            blocked: fraud.blocked,
            bypassed: fraud.bypassed,
            raw: Some(serde_json::to_value(fraud).unwrap_or_default()),
        }
    }
}

#[async_trait]
impl FraudDetector for RadarFraudDetector {
    async fn check_fraud(&self, options: &FraudCheckOptions) -> EveryMapResult<FraudResult> {
        let mut body = serde_json::Map::new();

        body.insert("deviceId".to_string(), serde_json::Value::String(options.device_id.clone()));
        body.insert("latitude".to_string(), serde_json::json!(options.latitude));
        body.insert("longitude".to_string(), serde_json::json!(options.longitude));
        body.insert("accuracy".to_string(), serde_json::json!(options.accuracy));

        if let Some(uid) = &options.user_id {
            body.insert("userId".to_string(), serde_json::Value::String(uid.clone()));
        }
        if let Some(fg) = options.foreground {
            body.insert("foreground".to_string(), serde_json::Value::Bool(fg));
        }
        if let Some(stopped) = options.stopped {
            body.insert("stopped".to_string(), serde_json::Value::Bool(stopped));
        }
        if let Some(dt) = &options.device_type {
            body.insert("deviceType".to_string(), serde_json::Value::String(dt.clone()));
        }
        if let Some(metadata) = &options.metadata {
            body.insert("metadata".to_string(), serde_json::to_value(metadata).unwrap_or_default());
        }

        let builder = self.client
            .build_request(reqwest::Method::POST, &self.base_url)
            .json(&serde_json::Value::Object(body));

        let radar_res: RadarTrackResponse = self.client.request_json(builder).await?;

        if radar_res.meta.code != 200 {
            return Err(EveryMapError::provider(
                "radar",
                radar_res.meta.code.to_string(),
                format!("Track/fraud check failed with status {}", radar_res.meta.code),
            ));
        }

        let fraud = radar_res.user.fraud.unwrap_or_default();

        Ok(FraudResult::from(fraud))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_fraud_data_all_clean() {
        let fraud = RadarFraudData {
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
        };
        let result: FraudResult = fraud.into();

        assert!(result.verified);
        assert!(result.passed);
        assert!(!result.mocked);
        assert!(!result.jumped);
        assert!(!result.compromised);
        assert!(!result.inaccurate);
        assert!(!result.proxy);
        assert!(!result.sharing);
        assert!(!result.blocked);
        assert!(!result.bypassed);
        assert!(result.raw.is_some());
    }

    #[test]
    fn test_radar_fraud_data_all_fraud_flags() {
        let fraud = RadarFraudData {
            verified: false,
            passed: false,
            mocked: true,
            jumped: true,
            compromised: true,
            inaccurate: true,
            proxy: true,
            sharing: true,
            blocked: true,
            bypassed: true,
        };
        let result: FraudResult = fraud.into();

        assert!(!result.verified);
        assert!(!result.passed);
        assert!(result.mocked);
        assert!(result.jumped);
        assert!(result.compromised);
        assert!(result.inaccurate);
        assert!(result.proxy);
        assert!(result.sharing);
        assert!(result.blocked);
        assert!(result.bypassed);
    }

    #[test]
    fn test_radar_fraud_data_default() {
        let fraud = RadarFraudData::default();
        let result: FraudResult = fraud.into();

        // Default RadarFraudData has all fields false
        assert!(!result.verified);
        assert!(!result.passed);
        assert!(!result.mocked);
    }

    #[test]
    fn test_radar_fraud_data_partial_flags() {
        let fraud = RadarFraudData {
            verified: true,
            passed: false,
            mocked: true,
            jumped: false,
            compromised: false,
            inaccurate: true,
            proxy: true,
            sharing: false,
            blocked: false,
            bypassed: false,
        };
        let result: FraudResult = fraud.into();

        assert!(result.verified);
        assert!(!result.passed);
        assert!(result.mocked);
        assert!(!result.jumped);
        assert!(result.inaccurate);
        assert!(result.proxy);
    }
}