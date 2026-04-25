use async_trait::async_trait;
use everymap_core::error::EveryMapResult;

/// Extension trait for HERE-specific geocoder capabilities.
///
/// These methods are not part of the core `Geocoder` trait because they
/// are HERE-specific. Import this trait to access them:
///
/// ```ignore
/// use everymap_providers_here::HereGeocoderExt;
/// let results = geocoder.discover(query, &options).await?;
/// ```
#[async_trait]
pub trait HereGeocoderExt: Send + Sync {
    /// Discover places/POIs matching a query.
    async fn discover(
        &self,
        req: super::domain::search::DiscoverRequest,
    ) -> EveryMapResult<super::domain::search::HereDiscoverResponse>;

    /// Get autosuggest results for a partial query (type-ahead).
    async fn autosuggest(
        &self,
        req: super::domain::search::AutosuggestRequest,
    ) -> EveryMapResult<super::domain::search::HereAutosuggestResponse>;
}

/// Extension trait for HERE-specific traffic capabilities.
#[async_trait]
pub trait HereTrafficExt: Send + Sync {
    /// Get detailed traffic flow data.
    async fn get_flow(
        &self,
        location: everymap_core::types::Coordinate,
        options: &super::domain::traffic::HereFlowOptions,
    ) -> EveryMapResult<super::domain::traffic::HereFlowResponse>;

    /// Get traffic incidents in an area.
    async fn get_incidents(
        &self,
        options: &super::domain::traffic::HereIncidentsOptions,
    ) -> EveryMapResult<super::domain::traffic::HereIncidentsResponse>;
}

/// Extension trait for HERE-specific positioning capabilities.
#[async_trait]
pub trait HerePositionerExt: Send + Sync {
    /// Get a position estimate using WiFi/cell/Bluetooth observations.
    async fn locate(
        &self,
        options: super::domain::positioning::HerePositioningOptions,
    ) -> EveryMapResult<super::domain::positioning::PositioningResponse>;
}

/// Extension trait for HERE-specific tour planning capabilities.
#[async_trait]
pub trait HereTourPlannerExt: Send + Sync {
    /// Solve a tour planning problem synchronously.
    async fn solve(
        &self,
        problem: super::domain::tour::TourProblem,
    ) -> EveryMapResult<super::domain::tour::TourSolution>;

    /// Submit a tour planning problem for asynchronous solving.
    async fn solve_async(
        &self,
        problem: super::domain::tour::TourProblem,
    ) -> EveryMapResult<super::domain::tour::AsyncSubmissionResult>;

    /// Get the status of an async tour planning job.
    async fn get_async_status(
        &self,
        status_id: &str,
    ) -> EveryMapResult<super::domain::tour::AsyncJobStatus>;

    /// Get the solution for a completed async tour planning job.
    async fn get_solution(
        &self,
        problem_id: &str,
    ) -> EveryMapResult<super::domain::tour::TourSolution>;

    /// Cancel an async tour planning job.
    async fn cancel(
        &self,
        problem_id: &str,
    ) -> EveryMapResult<super::domain::tour::CancellationStatus>;

    /// Get the version of the tour planning API.
    async fn version(&self) -> EveryMapResult<super::domain::tour::VersionResponse>;

    /// Check the health of the tour planning API.
    async fn health(&self) -> EveryMapResult<super::domain::tour::HealthResponse>;
}

/// Extension trait for HERE-specific attribute capabilities.
///
/// Provides typed access to the Map Attributes API v8 layers.
#[async_trait]
pub trait HereAttributeExt: Send + Sync {
    /// Get road attributes for a bounding box.
    async fn get_road_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse>;

    /// Get segment (topology) attributes for a bounding box.
    async fn get_segment_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<super::domain::attributes::HereSegmentAttributesResponse>;

    /// Get administrative area attributes for a bounding box.
    async fn get_admin_areas(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereAdminAreasResponse>;

    /// Get building attributes for a bounding box.
    async fn get_buildings(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereBuildingsResponse>;

    /// Get landmark attributes for a bounding box.
    async fn get_landmarks(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereLandmarksResponse>;

    /// Get road attributes by specific feature IDs.
    async fn get_road_attributes_by_ids(
        &self,
        ids: &[String],
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse>;

    /// Get speed limits for a bounding area (convenience method).
    async fn get_speed_limits(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse>;
}

// --- Extension trait implementations ---

#[async_trait]
impl HereGeocoderExt for super::domain::search::HereGeocoder {
    async fn discover(
        &self,
        req: super::domain::search::DiscoverRequest,
    ) -> EveryMapResult<super::domain::search::HereDiscoverResponse> {
        self.discover(req).await
    }

    async fn autosuggest(
        &self,
        req: super::domain::search::AutosuggestRequest,
    ) -> EveryMapResult<super::domain::search::HereAutosuggestResponse> {
        self.autosuggest(req).await
    }
}

#[async_trait]
impl HereTrafficExt for super::domain::traffic::HereTraffic {
    async fn get_flow(
        &self,
        location: everymap_core::types::Coordinate,
        options: &super::domain::traffic::HereFlowOptions,
    ) -> EveryMapResult<super::domain::traffic::HereFlowResponse> {
        self.get_flow(location, options).await
    }

    async fn get_incidents(
        &self,
        options: &super::domain::traffic::HereIncidentsOptions,
    ) -> EveryMapResult<super::domain::traffic::HereIncidentsResponse> {
        self.get_incidents(options).await
    }
}

#[async_trait]
impl HerePositionerExt for super::domain::positioning::HerePositioner {
    async fn locate(
        &self,
        options: super::domain::positioning::HerePositioningOptions,
    ) -> EveryMapResult<super::domain::positioning::PositioningResponse> {
        self.locate(options).await
    }
}

#[async_trait]
impl HereTourPlannerExt for super::domain::tour::HereTourPlanner {
    async fn solve(
        &self,
        problem: super::domain::tour::TourProblem,
    ) -> EveryMapResult<super::domain::tour::TourSolution> {
        self.solve(problem).await
    }

    async fn solve_async(
        &self,
        problem: super::domain::tour::TourProblem,
    ) -> EveryMapResult<super::domain::tour::AsyncSubmissionResult> {
        self.solve_async(problem).await
    }

    async fn get_async_status(
        &self,
        status_id: &str,
    ) -> EveryMapResult<super::domain::tour::AsyncJobStatus> {
        self.get_async_status(status_id).await
    }

    async fn get_solution(
        &self,
        problem_id: &str,
    ) -> EveryMapResult<super::domain::tour::TourSolution> {
        self.get_solution(problem_id).await
    }

    async fn cancel(
        &self,
        problem_id: &str,
    ) -> EveryMapResult<super::domain::tour::CancellationStatus> {
        self.cancel(problem_id).await
    }

    async fn version(&self) -> EveryMapResult<super::domain::tour::VersionResponse> {
        self.version().await
    }

    async fn health(&self) -> EveryMapResult<super::domain::tour::HealthResponse> {
        self.health().await
    }
}

#[async_trait]
impl HereAttributeExt for super::domain::attributes::HereAttributeProvider {
    async fn get_road_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse> {
        self.get_road_attributes(bbox, includes).await
    }

    async fn get_segment_attributes(
        &self,
        bbox: &str,
        includes: Option<Vec<String>>,
    ) -> EveryMapResult<super::domain::attributes::HereSegmentAttributesResponse> {
        self.get_segment_attributes(bbox, includes).await
    }

    async fn get_admin_areas(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereAdminAreasResponse> {
        self.get_admin_areas(bbox).await
    }

    async fn get_buildings(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereBuildingsResponse> {
        self.get_buildings(bbox).await
    }

    async fn get_landmarks(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereLandmarksResponse> {
        self.get_landmarks(bbox).await
    }

    async fn get_road_attributes_by_ids(
        &self,
        ids: &[String],
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse> {
        self.get_road_attributes_by_ids(ids).await
    }

    async fn get_speed_limits(
        &self,
        bbox: &str,
    ) -> EveryMapResult<super::domain::attributes::HereRoadAttributesResponse> {
        self.get_speed_limits(bbox).await
    }
}
