pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::positioning::{
    NetworkPositioner as NetworkPositionerTrait, PositioningOptions,
    PositioningResponse as CorePositioningResponse,
};
use everymap_core::error::EveryMapResult;
use serde::Serialize;
use std::sync::Arc;

pub use types::*;

const POSITIONING_BASE_URL: &str = "https://positioning.hereapi.com/v2";

/// Internal request body for the Positioning API.
#[derive(Debug, Serialize)]
struct PositionRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    wlan: Option<Vec<WlanAccessPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cell: Option<Vec<CellTower>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bluetooth: Option<Vec<BluetoothBeacon>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback: Option<Fallback>,
}

impl From<HerePositioningOptions> for PositionRequestBody {
    fn from(options: HerePositioningOptions) -> Self {
        Self {
            wlan: options.wlan,
            cell: options.cell,
            bluetooth: options.bluetooth,
            fallback: options.fallback,
        }
    }
}

/// Implementation of NetworkPositioner for HERE Technologies.
pub struct HerePositioner {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HerePositioner {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: POSITIONING_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Get position estimate with rich response type.
    /// POST /v2/position
    pub async fn locate(
        &self,
        options: HerePositioningOptions,
    ) -> EveryMapResult<PositioningResponse> {
        let url = format!("{}/position", self.base_url);
        let body = PositionRequestBody::from(options);
        let builder = self
            .client
            .build_request(reqwest::Method::POST, &url)
            .json(&body);

        let result: PositioningResponse = self.client.request_json(builder).await?;
        Ok(result)
    }
}

/// Convert core `PositioningOptions` to HERE-specific `HerePositioningOptions`,
/// extracting fields from `provider_extra`.
fn positioning_options_from_core(options: &PositioningOptions) -> HerePositioningOptions {
    let mut here_options = HerePositioningOptions::default();

    if let Some(extra) = &options.provider_extra {
        if let Ok(parsed) = serde_json::from_value::<HerePositioningOptions>(extra.clone()) {
            here_options = parsed;
        }
    }

    here_options
}

impl From<PositioningResponse> for CorePositioningResponse {
    fn from(response: PositioningResponse) -> Self {
        let coordinate =
            everymap_core::types::Coordinate::new(response.location.lat, response.location.lng)
                .unwrap_or(everymap_core::types::Coordinate::ORIGIN);
        Self {
            coordinate,
            accuracy: response.location.accuracy,
            altitude: response.altitude.as_ref().and_then(|a| a.value),
            altitude_accuracy: response.altitude.as_ref().and_then(|a| a.accuracy),
            raw: None,
        }
    }
}

#[async_trait]
impl NetworkPositionerTrait for HerePositioner {
    async fn get_position(
        &self,
        options: &PositioningOptions,
    ) -> EveryMapResult<CorePositioningResponse> {
        let here_options = positioning_options_from_core(options);
        let result = self.locate(here_options).await?;
        Ok(result.into())
    }
}
