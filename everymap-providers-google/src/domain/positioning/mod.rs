pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::positioning::{
    NetworkPositioner, PositioningOptions, PositioningResponse as CorePositioningResponse,
};
use everymap_core::error::EveryMapResult;
use everymap_core::types::Coordinate;
use serde::Serialize;
use std::sync::Arc;

pub use types::*;

const GEOLOCATION_BASE_URL: &str = "https://www.googleapis.com/geolocation/v1";

/// Internal request body for the Geolocation API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeolocationRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    consider_ip: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wifi_access_points: Option<Vec<GoogleWifiAccessPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cell_towers: Option<Vec<GoogleCellTower>>,
}

impl From<GooglePositioningOptions> for GeolocationRequestBody {
    fn from(options: GooglePositioningOptions) -> Self {
        Self {
            consider_ip: options.consider_ip,
            wifi_access_points: options.wifi_access_points,
            cell_towers: options.cell_towers,
        }
    }
}

/// Implementation of NetworkPositioner for Google Geolocation API.
pub struct GooglePositioner {
    pub(crate) client: Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GooglePositioner {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: GEOLOCATION_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Get position estimate with rich response type.
    /// POST /geolocation/v1/geolocate
    pub async fn locate(
        &self,
        options: GooglePositioningOptions,
    ) -> EveryMapResult<GoogleGeolocationResponse> {
        let url = format!("{}/geolocate", self.base_url);
        let body = GeolocationRequestBody::from(options);
        let builder = self
            .client
            .build_request(reqwest::Method::POST, &url)
            .json(&body);

        let result: GoogleGeolocationResponse = self.client.request_json(builder).await?;
        Ok(result)
    }
}

/// Convert core `PositioningOptions` to Google-specific `GooglePositioningOptions`,
/// extracting fields from `provider_extra`.
fn positioning_options_from_core(options: &PositioningOptions) -> GooglePositioningOptions {
    let mut google_opts = GooglePositioningOptions::default();

    if let Some(extra) = &options.provider_extra {
        if let Ok(parsed) = serde_json::from_value::<GooglePositioningOptions>(extra.clone()) {
            google_opts = parsed;
        }
    }

    google_opts
}

impl From<GoogleGeolocationResponse> for CorePositioningResponse {
    fn from(response: GoogleGeolocationResponse) -> Self {
        let coordinate = Coordinate::new(response.location.lat, response.location.lng)
            .unwrap_or(Coordinate::ORIGIN);
        Self {
            coordinate,
            accuracy: if response.accuracy > 0.0 {
                Some(response.accuracy)
            } else {
                None
            },
            altitude: None,
            altitude_accuracy: None,
            raw: None,
        }
    }
}

#[async_trait]
impl NetworkPositioner for GooglePositioner {
    async fn get_position(
        &self,
        options: &PositioningOptions,
    ) -> EveryMapResult<CorePositioningResponse> {
        let google_opts = positioning_options_from_core(options);
        let result = self.locate(google_opts).await?;
        Ok(result.into())
    }
}
