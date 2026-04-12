use serde::{Deserialize, Serialize};

// ============================================================================
// Request types — full coverage of HERE Tour Planning API v3
// ============================================================================

/// Top-level problem definition for the Tour Planning API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourProblem {
    pub fleet: Fleet,
    pub plan: Plan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Configuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectives: Option<Vec<Objective>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_objectives: Option<Vec<Vec<MultiObjective>>>,
}

/// Fleet definition: vehicle types and routing profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fleet {
    pub types: Vec<VehicleType>,
    pub profiles: Vec<Profile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic: Option<FleetTraffic>,
}

/// Fleet-level traffic setting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FleetTraffic {
    #[serde(rename = "liveOrHistorical")]
    LiveOrHistorical,
    #[serde(rename = "historicalOnly")]
    HistoricalOnly,
    #[default]
    #[serde(rename = "automatic")]
    Automatic,
}

/// Routing profile (discriminated by type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Profile {
    Car {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Truck {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Scooter {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Bicycle {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Pedestrian {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Bus {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    PrivateBus {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
    Taxi {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        departure_time: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traffic: Option<ProfileTraffic>,
    },
}

/// Profile-level traffic override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileTraffic {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "liveOrHistorical")]
    LiveOrHistorical,
    #[serde(rename = "historicalOnly")]
    HistoricalOnly,
    #[serde(rename = "automatic")]
    Automatic,
}

/// Vehicle type definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleType {
    pub id: String,
    pub profile: String,
    pub costs: VehicleCosts,
    pub shifts: Vec<VehicleShift>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub territory_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<VehicleLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Vehicle cost structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleCosts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<f64>,
}

/// Vehicle shift definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleShift {
    pub start: ShiftStart,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<ShiftEnd>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaks: Option<Vec<VehicleBreak>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reloads: Option<Vec<Reload>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_driving_time: Option<u32>,
}

/// Shift start.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShiftStart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TourLocation>,
}

/// Shift end.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShiftEnd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TourLocation>,
}

/// Vehicle break definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleBreak {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Vec<String>>>,
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TourLocation>,
}

/// Reload (capacity restore) point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reload {
    pub location: TourLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Vec<String>>>,
}

/// Vehicle limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_distance: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift_time: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_stops: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_stops: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_driving_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_on_vehicle: Option<u32>,
}

/// Plan definition: jobs, relations, clustering, groups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Plan {
    pub jobs: Vec<Job>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clustering: Option<PlanClustering>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<Group>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<Shared>,
}

/// Job definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub tasks: JobTasks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_on_vehicle: Option<MaxTimeOnVehicle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Job tasks: pickups and deliveries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobTasks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pickups: Option<Vec<JobTask>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deliveries: Option<Vec<JobTask>>,
}

/// A single job task (pickup or delivery).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobTask {
    pub places: Vec<JobPlace>,
    pub demand: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<JobPositionInTour>,
}

/// Job place with location, duration, and time windows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobPlace {
    pub location: TourLocation,
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_windows: Option<Vec<SoftTimeWindow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub house_key_id: Option<String>,
}

/// Soft time window for a job place.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftTimeWindow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<String>,
}

/// Job position constraint within a tour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPositionInTour {
    #[serde(rename = "first")]
    First,
    #[serde(rename = "last")]
    Last,
    #[serde(rename = "any")]
    Any,
}

/// Max time on vehicle constraint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaxTimeOnVehicle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

/// Relation between jobs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub jobs: Vec<String>,
    pub vehicle_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift_index: Option<u32>,
}

/// Relation type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum RelationType {
    #[default]
    #[serde(rename = "sequence")]
    Sequence,
    #[serde(rename = "flexible")]
    Flexible,
    #[serde(rename = "tour")]
    Tour,
}

/// Service duration clustering strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlanClustering {
    #[serde(rename = "maxDurationStrategy")]
    MaxDuration,
    #[serde(rename = "fixedDurationStrategy")]
    FixedDuration { duration: u32 },
    #[serde(rename = "boundedSumStrategy")]
    BoundedSum { maximum: u32 },
}

/// Job group (BETA).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pud_os: Option<Vec<Pudo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<GroupPlacement>,
}

/// Group placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupPlacement {
    #[serde(rename = "strict")]
    Strict,
    #[serde(rename = "flexible")]
    Flexible,
}

/// Pick-up/drop-off point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pudo {
    pub id: String,
    pub assign_at: PudoAssignAt,
    pub places: Vec<PudoPlace>,
}

/// PUDO assignment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum PudoAssignAt {
    #[default]
    #[serde(rename = "first")]
    First,
    #[serde(rename = "last")]
    Last,
}

/// PUDO place.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PudoPlace {
    pub location: TourLocation,
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_windows: Option<Vec<SoftTimeWindow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Shared resources (ALPHA).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Shared {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parking: Option<Vec<Parking>>,
}

/// Parking definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parking {
    pub id: String,
    pub places: Vec<ParkingPlace>,
}

/// Parking place.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParkingPlace {
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle_type_ids: Option<Vec<String>>,
}

/// Solver configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination: Option<Termination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_features: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_details: Option<RouteDetailsConfiguration>,
}

/// Termination settings for the solver.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Termination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stagnation_time: Option<u32>,
}

/// Route details configuration (currently just "polyline").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteDetailsConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<RouteDetailType>>,
}

/// Route detail type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteDetailType {
    #[serde(rename = "polyline")]
    Polyline,
}

/// Optimization objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Objective {
    #[serde(rename = "minimizeUnassigned")]
    MinimizeUnassigned,
    #[serde(rename = "minimizeCost")]
    MinimizeCost,
    #[serde(rename = "minimizeDuration")]
    MinimizeDuration,
    #[serde(rename = "minimizeDistance")]
    MinimizeDistance,
    #[serde(rename = "optimizeTaskOrder")]
    OptimizeTaskOrder,
    #[serde(rename = "optimizeTaskPosition")]
    OptimizeTaskPosition,
}

/// Multi-objective (ALPHA).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiObjective {
    #[serde(rename = "minimizeUnassigned")]
    MinimizeUnassigned,
    #[serde(rename = "minimizeCost")]
    MinimizeCost,
    #[serde(rename = "minimizeDuration")]
    MinimizeDuration,
    #[serde(rename = "minimizeDistance")]
    MinimizeDistance,
    #[serde(rename = "optimizeTaskOrder")]
    OptimizeTaskOrder,
    #[serde(rename = "optimizeTaskPosition")]
    OptimizeTaskPosition,
}

/// Location with lat/lng.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourLocation {
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lng: f64,
}

// ============================================================================
// Response types — full coverage of HERE Tour Planning API v3
// ============================================================================

/// Full solution response from the Tour Planning API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourSolution {
    #[serde(default)]
    pub statistic: TourStatistic,
    #[serde(default)]
    pub tours: Vec<TourTour>,
    #[serde(default)]
    pub unassigned: Vec<UnassignedJob>,
    #[serde(default)]
    pub notices: Vec<TourNotice>,
}

/// Overall solution statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourStatistic {
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub distance: f64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub times: Option<TourTimes>,
    #[serde(default)]
    pub stops: u32,
    #[serde(default)]
    pub tours: u32,
    #[serde(default, rename = "unassignedJobs")]
    pub unassigned_jobs: u32,
}

/// Time breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourTimes {
    #[serde(default)]
    pub driving: u32,
    #[serde(default)]
    pub serving: u32,
    #[serde(default)]
    pub waiting: u32,
    #[serde(rename = "break", default)]
    pub break_time: u32,
}

/// A single tour in the solution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourTour {
    #[serde(default, rename = "vehicleId")]
    pub vehicle_id: Option<String>,
    #[serde(default, rename = "typeId")]
    pub type_id: Option<String>,
    #[serde(default)]
    pub stops: Vec<TourStop>,
    #[serde(default)]
    pub statistic: TourStatistic,
}

/// A stop within a tour.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourStop {
    #[serde(default)]
    pub location: Option<TourLocation>,
    #[serde(default)]
    pub time: Option<TourStopTime>,
    #[serde(default)]
    pub activities: Vec<TourActivity>,
    #[serde(default)]
    pub distance: u32,
}

/// Stop arrival/departure times.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourStopTime {
    #[serde(default)]
    pub arrival: Option<String>,
    #[serde(default)]
    pub departure: Option<String>,
}

/// Activity at a stop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourActivity {
    #[serde(default, rename = "jobId")]
    pub job_id: Option<String>,
    #[serde(default, rename = "type")]
    pub activity_type: Option<ActivityType>,
    #[serde(default)]
    pub demand: Option<Vec<i64>>,
}

/// Activity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    #[serde(rename = "pickup")]
    Pickup,
    #[serde(rename = "delivery")]
    Delivery,
    #[serde(rename = "departure")]
    Departure,
    #[serde(rename = "arrival")]
    Arrival,
    #[serde(rename = "break")]
    Break,
    #[serde(rename = "reload")]
    Reload,
}

/// Unassigned job with reasons.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnassignedJob {
    #[serde(default, rename = "jobId")]
    pub job_id: Option<String>,
    #[serde(default)]
    pub reasons: Vec<UnassignedJobReason>,
}

/// Reason a job was unassigned.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnassignedJobReason {
    #[serde(default)]
    pub code: Option<UnassignmentReasonCode>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub details: Option<Vec<UnassignedJobDetail>>,
}

/// Unassignment reason code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnassignmentReasonCode {
    #[serde(rename = "NO_REASON_FOUND")]
    NoReasonFound,
    #[serde(rename = "SKILL_CONSTRAINT")]
    SkillConstraint,
    #[serde(rename = "TIME_WINDOW_CONSTRAINT")]
    TimeWindowConstraint,
    #[serde(rename = "CAPACITY_CONSTRAINT")]
    CapacityConstraint,
    #[serde(rename = "REACHABLE_CONSTRAINT")]
    ReachableConstraint,
    #[serde(rename = "MAX_DISTANCE_CONSTRAINT")]
    MaxDistanceConstraint,
    #[serde(rename = "SHIFT_TIME_CONSTRAINT")]
    ShiftTimeConstraint,
    #[serde(rename = "LOCKING_CONSTRAINT")]
    LockingConstraint,
    #[serde(rename = "TOUR_ORDER_CONSTRAINT")]
    TourOrderConstraint,
    #[serde(rename = "MAX_STOPS_CONSTRAINT")]
    MaxStopsConstraint,
    #[serde(rename = "MIN_STOPS_CONSTRAINT")]
    MinStopsConstraint,
    #[serde(rename = "BREAK_CONSTRAINT")]
    BreakConstraint,
    #[serde(rename = "AREA_CONSTRAINT")]
    AreaConstraint,
    #[serde(rename = "TERRITORY_CONSTRAINT")]
    TerritoryConstraint,
    #[serde(rename = "UNREACHABLE_IN_RELATION_CONSTRAINT")]
    UnreachableInRelationConstraint,
    #[serde(rename = "MIXING_RESTRICTION_CONSTRAINT")]
    MixingRestrictionConstraint,
    #[serde(rename = "GROUP_CONSTRAINT")]
    GroupConstraint,
    #[serde(rename = "MAX_DRIVING_TIME_CONSTRAINT")]
    MaxDrivingTimeConstraint,
    #[serde(rename = "MAX_TIME_ON_VEHICLE_CONSTRAINT")]
    MaxTimeOnVehicleConstraint,
}

/// Detail about an unassigned job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnassignedJobDetail {
    #[serde(default, rename = "vehicleIds")]
    pub vehicle_ids: Option<Vec<String>>,
    #[serde(default, rename = "shiftIndex")]
    pub shift_index: Option<u32>,
}

/// Notice from the solver.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TourNotice {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
}

// ============================================================================
// Async types
// ============================================================================

/// Async submission result (HTTP 202).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncSubmissionResult {
    #[serde(default, rename = "statusId")]
    pub status_id: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
}

/// Async job status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncJobStatus {
    #[serde(default)]
    pub status: Option<AsyncStatus>,
    #[serde(default)]
    pub resource: Option<AsyncResource>,
    #[serde(default)]
    pub error: Option<AsyncError>,
}

/// Async status values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "inProgress")]
    InProgress,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "canceled")]
    Canceled,
}

/// Resource reference in async status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncResource {
    #[serde(default, rename = "resourceId")]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
}

/// Async error.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncError {
    #[serde(default)]
    pub message: Option<String>,
}

/// Cancellation status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CancellationStatus {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub started: Option<String>,
    #[serde(default)]
    pub ended: Option<String>,
}

/// Version response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionResponse {
    #[serde(default, rename = "apiVersion")]
    pub api_version: Option<String>,
}

/// Health response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthResponse {
    #[serde(default)]
    pub status: Option<String>,
}