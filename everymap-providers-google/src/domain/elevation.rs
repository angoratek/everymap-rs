//! Google Maps Elevation API.
//!
//! Samples terrain elevation for ordered points through
//! `/maps/api/elevation/json`. The API accepts up to
//! [`MAX_ELEVATION_LOCATIONS_PER_REQUEST`] locations per request, so a longer
//! list is split into ordered batches and the samples are joined back in the
//! caller's order.

use crate::client::{endpoint_url, GoogleClient};
use crate::domain::geo::GoogleLatLng;
use crate::error_mapping::{invalid_response, status_error};
use async_trait::async_trait;
use everymap_core::domains::elevation::{ElevationProvider, ElevationSample};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::Method;
use everymap_core::types::Coordinate;
use serde::Deserialize;
use std::sync::Arc;

/// Service root of the Google Maps Elevation API.
const ELEVATION_SERVICE_ROOT: &str = "https://maps.googleapis.com";

/// Documented path of the Google Maps Elevation API.
const ELEVATION_PATH: &str = "/maps/api/elevation/json";

/// The Elevation API accepts at most this many locations in one request.
pub const MAX_ELEVATION_LOCATIONS_PER_REQUEST: usize = 512;

/// Separator the Elevation API uses between requested locations.
const LOCATIONS_SEPARATOR: &str = "|";

/// The Elevation API's answer: one result per requested location, in order.
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleElevationResponse {
    /// One elevation result per requested location, in the request's order.
    #[serde(default)]
    pub results: Vec<GoogleElevationResult>,
    /// Google's in-body status, `OK` on success.
    pub status: String,
    /// Google's explanation when `status` is not `OK`.
    #[serde(default)]
    pub error_message: Option<String>,
}

/// One sampled location.
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleElevationResult {
    /// The elevation above sea level, in meters.
    pub elevation: f64,
    /// The sampled position, echoed from the request.
    pub location: GoogleLatLng,
    /// The maximum distance, in meters, between the sampled data points.
    #[serde(default)]
    pub resolution: Option<f64>,
}

/// Implementation of `ElevationProvider` for the Google Maps Elevation API.
pub struct GoogleElevationProvider {
    client: Arc<GoogleClient>,
    service_root: String,
}

impl GoogleElevationProvider {
    /// Creates an elevation provider against Google's public service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            service_root: ELEVATION_SERVICE_ROOT.to_string(),
        }
    }

    /// Creates an elevation provider against a configurable service root.
    ///
    /// # Parameters
    /// - `client` — the shared Google client.
    /// - `service_root` — the scheme and host that serve the API, for example
    ///   `https://maps.googleapis.com` or a test server's origin. The
    ///   documented path is appended.
    pub fn with_base_url(client: Arc<GoogleClient>, service_root: String) -> Self {
        Self {
            client,
            service_root,
        }
    }

    /// The endpoint URL this provider sends requests to.
    fn endpoint(&self) -> String {
        endpoint_url(&self.service_root, ELEVATION_PATH)
    }
}

impl TryFrom<GoogleElevationResult> for ElevationSample {
    type Error = EveryMapError;

    fn try_from(result: GoogleElevationResult) -> Result<Self, Self::Error> {
        let coordinate = Coordinate::new(result.location.lat, result.location.lng)
            .map_err(|_| invalid_response("elevation result carries an invalid location"))?;
        Ok(Self {
            coordinate,
            elevation_meters: result.elevation,
        })
    }
}

#[async_trait]
impl ElevationProvider for GoogleElevationProvider {
    async fn elevation_points(
        &self,
        points: &[Coordinate],
    ) -> EveryMapResult<Vec<ElevationSample>> {
        if points.is_empty() {
            return Err(EveryMapError::ValidationError(
                "elevation requires at least one point".to_string(),
            ));
        }

        let mut samples = Vec::with_capacity(points.len());
        for batch in points.chunks(MAX_ELEVATION_LOCATIONS_PER_REQUEST) {
            let locations = batch
                .iter()
                .map(|coordinate| format!("{},{}", coordinate.lat, coordinate.lng))
                .collect::<Vec<_>>()
                .join(LOCATIONS_SEPARATOR);
            let request = self
                .client
                .build_request(Method::GET, &self.endpoint())
                .query(&[("locations", locations)]);
            let response: GoogleElevationResponse =
                self.client.request_parsed_json(request).await?;

            if response.status != "OK" {
                return Err(status_error(
                    &response.status,
                    response.error_message.as_deref(),
                ));
            }

            samples.extend(
                response
                    .results
                    .into_iter()
                    .map(ElevationSample::try_from)
                    .collect::<EveryMapResult<Vec<_>>>()?,
            );
        }

        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::{GoogleElevationResult, MAX_ELEVATION_LOCATIONS_PER_REQUEST};
    use crate::domain::geo::GoogleLatLng;
    use everymap_core::domains::elevation::ElevationSample;

    #[test]
    fn a_result_projects_onto_a_sample() {
        let sample = ElevationSample::try_from(GoogleElevationResult {
            elevation: 34.0,
            location: GoogleLatLng {
                lat: 52.5,
                lng: 13.4,
            },
            resolution: Some(19.0),
        })
        .expect("a valid result projects");
        assert_eq!(sample.elevation_meters, 34.0);
        assert_eq!(sample.coordinate.lat, 52.5);
        assert_eq!(sample.coordinate.lng, 13.4);
    }

    #[test]
    fn the_documented_batch_limit_is_a_named_constant() {
        assert_eq!(MAX_ELEVATION_LOCATIONS_PER_REQUEST, 512);
    }
}
