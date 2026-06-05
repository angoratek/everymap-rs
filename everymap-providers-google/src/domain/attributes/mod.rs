pub mod types;

use crate::client::GoogleClient;
use async_trait::async_trait;
use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider, AttributeResponse};
use everymap_core::error::{EveryMapError, EveryMapResult};
use std::sync::Arc;

pub use types::*;

const ROADS_BASE_URL: &str = "https://roads.googleapis.com/v1";

/// Implementation of AttributeProvider for Google Roads API speedLimits.
///
/// Google's attribute coverage is limited to speed limits via the Roads API.
/// For richer attribute data (road class, lanes, etc.), use HERE or TomTom.
pub struct GoogleAttributeProvider {
    pub(crate) client: Arc<GoogleClient>,
    pub(crate) base_url: String,
}

impl GoogleAttributeProvider {
    pub fn new(client: Arc<GoogleClient>) -> Self {
        Self {
            client,
            base_url: ROADS_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<GoogleClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Get speed limits by place IDs.
    /// GET /v1/speedLimits?placeId=...&placeId=...&units=...
    pub async fn get_speed_limits_by_ids(
        &self,
        place_ids: &[String],
        units: Option<&str>,
    ) -> EveryMapResult<GoogleSpeedLimitsResponse> {
        let url = format!("{}/speedLimits", self.base_url);
        let mut params: Vec<(&str, String)> = Vec::new();

        for place_id in place_ids {
            params.push(("placeId", place_id.clone()));
        }

        if let Some(u) = units {
            params.push(("units", u.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: GoogleSpeedLimitsResponse = self.client.request_json(builder).await?;
        Ok(result)
    }

    /// Get speed limits along a path (snapped to roads).
    /// GET /v1/speedLimits?path=...&units=...
    pub async fn get_speed_limits_along_path(
        &self,
        path: &str,
        units: Option<&str>,
    ) -> EveryMapResult<GoogleSpeedLimitsResponse> {
        let url = format!("{}/speedLimits", self.base_url);
        let mut params: Vec<(&str, String)> = vec![("path", path.to_string())];

        if let Some(u) = units {
            params.push(("units", u.to_string()));
        }

        let builder = self
            .client
            .build_request(reqwest::Method::GET, &url)
            .query(&params);

        let result: GoogleSpeedLimitsResponse = self.client.request_json(builder).await?;
        Ok(result)
    }
}

/// Convert core `AttributeOptions` to Google-specific parameters.
fn attribute_options_from_core(
    options: &AttributeOptions,
) -> EveryMapResult<GoogleAttributeOptions> {
    let mut google_opts = GoogleAttributeOptions::default();

    if options.bbox.is_some() {
        log::warn!(
            "Google Roads API speedLimits does not support bbox queries; \
             use provider_extra.place_ids or provider_extra.path instead"
        );
    }
    if options.language.is_some() {
        log::warn!(
            "Google Roads API speedLimits does not support language parameter; ignoring"
        );
    }

    if let Some(extra) = &options.provider_extra {
        if let Some(obj) = extra.as_object() {
            if let Some(v) = obj.get("place_ids").and_then(|v| v.as_array()) {
                let ids: Vec<String> = v
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                google_opts.place_ids = Some(ids);
            }
            if let Some(v) = obj.get("path").and_then(|v| v.as_str()) {
                google_opts.path = Some(v.to_string());
            }
            if let Some(v) = obj.get("units").and_then(|v| v.as_str()) {
                google_opts.units = Some(v.to_string());
            }
        }
    }

    // Require either place_ids or path
    if google_opts.place_ids.is_none() && google_opts.path.is_none() {
        // Default to a place_id from bbox if provided (uncommon for Google)
        return Err(EveryMapError::provider(
            "google",
            "MISSING_PARAMETER",
            "Google AttributeProvider requires either 'place_ids' or 'path' in provider_extra",
        ));
    }

    Ok(google_opts)
}

#[async_trait]
impl AttributeProvider for GoogleAttributeProvider {
    async fn get_attributes(
        &self,
        options: &AttributeOptions,
    ) -> EveryMapResult<AttributeResponse> {
        let google_opts = attribute_options_from_core(options)?;

        let result = if let Some(place_ids) = &google_opts.place_ids {
            self.get_speed_limits_by_ids(place_ids, google_opts.units.as_deref())
                .await?
        } else if let Some(path) = &google_opts.path {
            self.get_speed_limits_along_path(path, google_opts.units.as_deref())
                .await?
        } else {
            return Err(EveryMapError::provider(
                "google",
                "MISSING_PARAMETER",
                "Google AttributeProvider requires either 'place_ids' or 'path' in provider_extra",
            ));
        };

        let data = serde_json::to_value(result).unwrap_or(serde_json::json!({}));

        Ok(AttributeResponse { data })
    }
}
