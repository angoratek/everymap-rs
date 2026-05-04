pub mod types;

use crate::client::HereClient;
use async_trait::async_trait;
use everymap_core::domains::tour::{
    TourOptions, TourPlanner, TourResponse, TourStop as CoreTourStop,
};
use everymap_core::error::EveryMapResult;
use std::sync::Arc;

pub use types::*;

use everymap_core::types::Coordinate;

const TOUR_BASE_URL: &str = "https://tourplanning.hereapi.com/v3";

impl From<TourSolution> for TourResponse {
    fn from(solution: TourSolution) -> Self {
        let tour_stops: Vec<CoreTourStop> = solution
            .tours
            .first()
            .map(|tour| {
                tour.stops
                    .iter()
                    .filter_map(|s| {
                        s.location
                            .as_ref()
                            .map(|loc| {
                                Coordinate::new(loc.lat, loc.lng).unwrap_or(Coordinate::ORIGIN)
                            })
                            .map(|coordinate| CoreTourStop {
                                coordinate,
                                arrival_time: s.time.as_ref().and_then(|t| t.arrival.clone()),
                                departure_time: None,
                                duration: None,
                                distance_from_previous: None,
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self {
            stops: tour_stops,
            total_distance: Some(solution.statistic.distance),
            total_duration: Some(solution.statistic.duration),
            unassigned_count: Some(solution.unassigned.len() as u32),
            raw: None,
        }
    }
}

/// Exhaustive options for HERE Tour Planning API v3.
/// Wraps the full `TourProblem` for the POST body.
#[derive(Debug, Clone, Default)]
pub struct HereTourOptions {
    pub problem: TourProblem,
}

/// Implementation of TourPlanner for HERE Technologies.
pub struct HereTourPlanner {
    pub(crate) client: Arc<HereClient>,
    pub(crate) base_url: String,
}

impl HereTourPlanner {
    pub fn new(client: Arc<HereClient>) -> Self {
        Self {
            client,
            base_url: TOUR_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(client: Arc<HereClient>, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Solve a tour planning problem synchronously (POST /problems).
    /// Returns the full rich response.
    pub async fn solve(&self, problem: TourProblem) -> EveryMapResult<TourSolution> {
        let url = format!("{}/problems", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::POST, &url)
            .json(&problem);

        let solution: TourSolution = self.client.request_json(builder).await?;
        Ok(solution)
    }

    /// Submit a tour planning problem asynchronously (POST /problems/async).
    /// Returns the async submission result with status ID.
    pub async fn solve_async(&self, problem: TourProblem) -> EveryMapResult<AsyncSubmissionResult> {
        let url = format!("{}/problems/async", self.base_url);
        let builder = self
            .client
            .build_request(reqwest::Method::POST, &url)
            .json(&problem);

        let result: AsyncSubmissionResult = self.client.request_json(builder).await?;
        Ok(result)
    }

    /// Get async job status (GET /status/{statusId}).
    pub async fn get_async_status(&self, status_id: &str) -> EveryMapResult<AsyncJobStatus> {
        let url = format!("{}/status/{}", self.base_url, status_id);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let status: AsyncJobStatus = self.client.request_json(builder).await?;
        Ok(status)
    }

    /// Get solution for an async problem (GET /problems/{problemId}/solution).
    pub async fn get_solution(&self, problem_id: &str) -> EveryMapResult<TourSolution> {
        let url = format!("{}/problems/{}/solution", self.base_url, problem_id);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let solution: TourSolution = self.client.request_json(builder).await?;
        Ok(solution)
    }

    /// Cancel an async problem (PUT /problems/{problemId}/cancel).
    pub async fn cancel(&self, problem_id: &str) -> EveryMapResult<CancellationStatus> {
        let url = format!("{}/problems/{}/cancel", self.base_url, problem_id);
        let builder = self.client.build_request(reqwest::Method::PUT, &url);

        let status: CancellationStatus = self.client.request_json(builder).await?;
        Ok(status)
    }

    /// Get API version (GET /version).
    pub async fn version(&self) -> EveryMapResult<VersionResponse> {
        let url = format!("{}/version", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let version: VersionResponse = self.client.request_json(builder).await?;
        Ok(version)
    }

    /// Health check (GET /health).
    pub async fn health(&self) -> EveryMapResult<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let builder = self.client.build_request(reqwest::Method::GET, &url);

        let health: HealthResponse = self.client.request_json(builder).await?;
        Ok(health)
    }
}

#[async_trait]
impl TourPlanner for HereTourPlanner {
    async fn optimize_tour(
        &self,
        stops: &[everymap_core::types::Coordinate],
        options: &TourOptions,
    ) -> EveryMapResult<TourResponse> {
        // Extract problem from provider_extra or create a default one
        let mut problem = options
            .provider_extra
            .as_ref()
            .and_then(|extra| serde_json::from_value::<TourProblem>(extra.clone()).ok())
            .unwrap_or_default();

        // If the plan has no jobs but stops were provided, create simple delivery jobs
        if problem.plan.jobs.is_empty() && !stops.is_empty() {
            problem.plan.jobs = stops
                .iter()
                .enumerate()
                .map(|(i, coordinate)| Job {
                    id: format!("stop_{}", i),
                    tasks: JobTasks {
                        deliveries: Some(vec![JobTask {
                            places: vec![JobPlace {
                                location: TourLocation {
                                    lat: coordinate.lat,
                                    lng: coordinate.lng,
                                },
                                duration: 60,
                                ..Default::default()
                            }],
                            demand: vec![1],
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .collect();
        }

        // Ensure fleet has at least one vehicle type if not provided
        if problem.fleet.types.is_empty() {
            let start_location = stops.first().map(|s| TourLocation {
                lat: s.lat,
                lng: s.lng,
            });
            let departure_time = options
                .provider_extra
                .as_ref()
                .and_then(|e| e.get("departure_time"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
            problem.fleet.types = vec![VehicleType {
                id: "vehicle_1".to_string(),
                profile: "car_profile".to_string(),
                costs: VehicleCosts {
                    fixed: Some(0.0),
                    distance: Some(1.0),
                    time: Some(0.0),
                    job: None,
                },
                shifts: vec![VehicleShift {
                    start: ShiftStart {
                        time: Some(departure_time),
                        earliest: None,
                        location: start_location,
                    },
                    ..Default::default()
                }],
                capacity: Some(vec![10]),
                amount: Some(1),
                ..Default::default()
            }];
        }

        if problem.fleet.profiles.is_empty() {
            problem.fleet.profiles = vec![Profile::Car {
                name: "car_profile".to_string(),
                departure_time: None,
                traffic: None,
            }];
        }

        let solution = self.solve(problem).await?;

        Ok(solution.into())
    }
}
