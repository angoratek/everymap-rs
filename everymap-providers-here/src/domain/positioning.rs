pub mod types;

use async_trait::async_trait;
use everymap_core::domains::positioning::{NetworkPositioner as NetworkPositionerTrait, PositioningRequest, PositioningResponse as CorePositioningResponse};
use everymap_core::error::EveryMapResult;
use crate::client::HereClient;
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
    fn from(opts: HerePositioningOptions) -> Self {
        Self {
            wlan: opts.wlan,
            cell: opts.cell,
            bluetooth: opts.bluetooth,
            fallback: opts.fallback,
        }
    }
}

/// Implementation of NetworkPositioner for HERE Technologies.
pub struct HerePositioner {
    client: Arc<HereClient>,
    base_url: String,
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
    pub async fn locate(&self, options: HerePositioningOptions) -> EveryMapResult<PositioningResponse> {
        let url = format!("{}/position", self.base_url);
        let body = PositionRequestBody::from(options);
        let builder = self.client.build_request(reqwest::Method::POST, &url)
            .json(&body);

        let response = self.client.request(builder).await?;
        let result: PositioningResponse = response.json().await?;
        Ok(result)
    }
}

#[async_trait]
impl NetworkPositionerTrait for HerePositioner {
    type Options = HerePositioningOptions;
    type Response = CorePositioningResponse;

    async fn get_position(&self, req: PositioningRequest<Self::Options>) -> EveryMapResult<Self::Response> {
        let result = self.locate(req.options).await?;
        let coordinate = everymap_core::types::Coordinate::new(
            result.location.lat,
            result.location.lng,
        ).map_err(|e| everymap_core::error::EveryMapError::ValidationError(e.to_string()))?;

        Ok(CorePositioningResponse {
            coordinate,
            accuracy: result.location.accuracy,
            altitude: result.altitude.as_ref().and_then(|a| a.value),
            altitude_accuracy: result.altitude.as_ref().and_then(|a| a.accuracy),
            raw: None,
        })
    }
}