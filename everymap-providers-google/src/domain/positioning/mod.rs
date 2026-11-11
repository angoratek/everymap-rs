pub mod types;

use crate::client::{endpoint_url, GoogleClient};
use crate::error_mapping::invalid_response;
use async_trait::async_trait;
use everymap_core::domains::positioning::{
    NetworkPositioner, PositioningOptions, PositioningResponse as CorePositioningResponse,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;
use serde::Serialize;
use std::sync::Arc;

pub use types::*;

/// Service root of the Google Geolocation API.
const GEOLOCATION_SERVICE_ROOT: &str = "https://www.googleapis.com";

/// Documented path of the Google Geolocation API.
const GEOLOCATION_PATH: &str = "/geolocation/v1/geolocate";

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
    pub(crate) service_root: String,
}

impl GooglePositioner {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            service_root: GEOLOCATION_SERVICE_ROOT.to_string(),
        }
    }

    /// Creates a positioner against a Google service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    /// - `service_root` — the scheme and host that serve the API, for example
    ///   `https://www.googleapis.com` or a test server's origin. The documented
    ///   path is appended.
    pub fn with_base_url(client: Arc<GoogleClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// Get position estimate with rich response type.
    /// POST /geolocation/v1/geolocate
    pub async fn locate(
        &self,
        options: GooglePositioningOptions,
    ) -> EveryMapResult<GoogleGeolocationResponse> {
        let url = endpoint_url(&self.service_root, GEOLOCATION_PATH);
        let body = GeolocationRequestBody::from(options);
        let builder = self
            .client
            .build_request(everymap_core::http::Method::POST, &url)
            .json(&body)?;

        let result: GoogleGeolocationResponse = self.client.request_parsed_json(builder).await?;
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

impl TryFrom<GoogleGeolocationResponse> for CorePositioningResponse {
    type Error = EveryMapError;

    fn try_from(response: GoogleGeolocationResponse) -> Result<Self, Self::Error> {
        let latitude = response
            .location
            .lat
            .ok_or_else(|| invalid_response("position estimate lacks a latitude"))?;
        let longitude = response
            .location
            .lng
            .ok_or_else(|| invalid_response("position estimate lacks a longitude"))?;
        let coordinate = Coordinate::new(latitude, longitude)
            .map_err(|_| invalid_response("position estimate carries an invalid location"))?;
        Ok(Self {
            coordinate,
            accuracy: if response.accuracy > 0.0 {
                Some(response.accuracy)
            } else {
                None
            },
            altitude: None,
            altitude_accuracy: None,
            raw: None,
        })
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
        CorePositioningResponse::try_from(result)
    }
}
