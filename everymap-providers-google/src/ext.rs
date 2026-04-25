use async_trait::async_trait;
use everymap_core::error::EveryMapResult;

/// Extension trait for Google-specific positioning capabilities.
///
/// Provides access to the rich Google Geolocation API response type,
/// which includes more detail than the core `PositioningResponse`.
///
/// ```ignore
/// use everymap_providers_google::GooglePositionerExt;
/// let result = positioner.locate(google_options).await?;
/// ```
#[async_trait]
pub trait GooglePositionerExt: Send + Sync {
    /// Get a position estimate with rich Google response type.
    /// POST /geolocation/v1/geolocate
    async fn locate(
        &self,
        options: super::domain::positioning::GooglePositioningOptions,
    ) -> EveryMapResult<super::domain::positioning::GoogleGeolocationResponse>;
}

/// Extension trait for Google-specific attribute capabilities.
///
/// Provides typed access to the Roads API speedLimits endpoints,
/// which return more detail than the core `AttributeResponse`.
///
/// ```ignore
/// use everymap_providers_google::GoogleAttributeExt;
/// let limits = provider.get_speed_limits_by_ids(&place_ids, None).await?;
/// ```
#[async_trait]
pub trait GoogleAttributeExt: Send + Sync {
    /// Get speed limits by place IDs.
    /// GET /v1/speedLimits?placeId=...&placeId=...&units=...
    async fn get_speed_limits_by_ids(
        &self,
        place_ids: &[String],
        units: Option<&str>,
    ) -> EveryMapResult<super::domain::attributes::GoogleSpeedLimitsResponse>;

    /// Get speed limits along a path (snapped to roads).
    /// GET /v1/speedLimits?path=...&units=...
    async fn get_speed_limits_along_path(
        &self,
        path: &str,
        units: Option<&str>,
    ) -> EveryMapResult<super::domain::attributes::GoogleSpeedLimitsResponse>;
}

// --- Extension trait implementations ---

#[async_trait]
impl GooglePositionerExt for super::domain::positioning::GooglePositioner {
    async fn locate(
        &self,
        options: super::domain::positioning::GooglePositioningOptions,
    ) -> EveryMapResult<super::domain::positioning::GoogleGeolocationResponse> {
        self.locate(options).await
    }
}

#[async_trait]
impl GoogleAttributeExt for super::domain::attributes::GoogleAttributeProvider {
    async fn get_speed_limits_by_ids(
        &self,
        place_ids: &[String],
        units: Option<&str>,
    ) -> EveryMapResult<super::domain::attributes::GoogleSpeedLimitsResponse> {
        self.get_speed_limits_by_ids(place_ids, units).await
    }

    async fn get_speed_limits_along_path(
        &self,
        path: &str,
        units: Option<&str>,
    ) -> EveryMapResult<super::domain::attributes::GoogleSpeedLimitsResponse> {
        self.get_speed_limits_along_path(path, units).await
    }
}
