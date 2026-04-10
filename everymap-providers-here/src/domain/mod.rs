pub mod geo;
pub mod search;
pub mod routing;
pub mod isoline;
pub mod matching;
pub mod tour;
pub mod traffic;
pub mod tiling;
pub mod positioning;
pub mod attributes;
pub mod imaging;

// Shared HERE types
pub use geo::HereLatLng;

// Explicit re-exports to avoid ambiguous glob conflicts
pub use search::{
    HereGeocoder, HereGeocodeOptions,
    DiscoverRequest, AutosuggestRequest,
    HereDiscoverOptions, HereAutosuggestOptions,
    HereDiscoverResponse, HereAutosuggestResponse, HereSearchItem, HereAddress,
    HereCategory, HereContact, HereFood, HereRating, HereMapView, HereChain,
    HereHighlights, HereHighlightSection, HereReference, HereAutosuggestItem, HereQueryTerm,
    SearchType, AddressNamesMode, PostalCodeMode, WithFeature, ShowFeature,
    ShowMapReference, ShowNavAttribute, ShowRelated, ShowTranslation,
    DiscoverWithFeature, MobilityMode, RankingMode, HereFuelStationFilter,
    HereEvStationFilter, HereEvSupplyType, HereRouteFilter,
};
pub use routing::{
    HereRouter, HereRouteOptions, TransportMode, RoutingMode, Units,
    ReturnField, ConsumptionModel, TrafficMode, ScooterParams, TruckParams,
    EvParams, FuelParams, DriverParams, TaxiParams, TollsParams, MaxSpeedOnSegment,
    HereRouteApiResponse, HereRoute, HereRouteSection, HereRouteSummary,
    HerePlace, HereRouteAction, HereTurnAction, HereNotice, HereSpan,
};
pub use isoline::{HereIsoline, HereIsolineOptions};
pub use matching::{HereRouteMatcher, HereMatchingOptions};
pub use tour::{
    HereTourPlanner, HereTourOptions,
    TourProblem, Fleet, FleetTraffic, Profile, ProfileTraffic,
    VehicleType, VehicleCosts, VehicleShift, ShiftStart, ShiftEnd,
    VehicleBreak, Reload, VehicleLimits,
    Plan, Job, JobTasks, JobTask, JobPlace, SoftTimeWindow,
    JobPositionInTour, MaxTimeOnVehicle, Relation, RelationType,
    PlanClustering, Group, GroupPlacement, Pudo, PudoAssignAt, PudoPlace,
    Shared, Parking, ParkingPlace,
    Configuration, Termination, RouteDetailsConfiguration, RouteDetailType,
    Objective, MultiObjective, TourLocation,
    TourSolution, TourStatistic, TourTimes, TourTour, TourStop,
    TourStopTime, TourActivity, ActivityType,
    UnassignedJob, UnassignedJobReason, UnassignmentReasonCode,
    UnassignedJobDetail, TourNotice,
    AsyncSubmissionResult, AsyncJobStatus, AsyncStatus, AsyncResource, AsyncError,
    CancellationStatus, VersionResponse, HealthResponse,
};
pub use traffic::{
    HereTraffic, HereFlowOptions, HereIncidentsOptions,
    HereFlowResponse, HereFlowItem, HereCurrentFlow, HereSubSegment, HereLane,
    HereRoadInfo, HereTrafficLocation, HereTrafficShapePoint,
    HereIncidentsResponse, HereIncidentItem, HereIncident, HereIncidentDescription,
    HereAffectedItem, LocationReferencing, AdvancedFeature, CriticalityLevel,
    IncidentType, TrafficUnits,
};
pub use tiling::{
    HereTileProvider, HereTileOptions, TileLayer, TileFormat,
};
pub use positioning::{
    HerePositioner, HerePositioningOptions,
    WlanAccessPoint, CellTower, BluetoothBeacon, RadioType, Fallback,
    PositioningResponse, PositionLocation, PositionAltitude,
};
pub use attributes::{
    HereAttributeProvider, HereAttributeOptions, AttributeLayer, AttributeFormat,
};
pub use imaging::{
    HereMapImageProvider, HereImageOptions, ImageFormat, MapStyle,
};