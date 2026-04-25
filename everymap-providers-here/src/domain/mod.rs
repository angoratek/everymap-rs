pub mod attributes;
pub mod geo;
pub mod imaging;
pub mod isoline;
pub mod matching;
pub mod positioning;
pub mod routing;
pub mod search;
pub mod tiling;
pub mod tour;
pub mod traffic;
pub mod unsupported;

// Shared HERE types
pub use geo::HereLatLng;

// Explicit re-exports to avoid ambiguous glob conflicts
pub use attributes::{
    AttributeFormat, AttributeLayer, HereAttributeOptions, HereAttributeProvider,
};
pub use imaging::{HereImageOptions, HereMapImageProvider, ImageFormat, MapStyle};
pub use isoline::{HereIsoline, HereIsolineOptions};
pub use matching::{HereMatchingOptions, HereRouteMatcher};
pub use positioning::{
    BluetoothBeacon, CellTower, Fallback, HerePositioner, HerePositioningOptions, PositionAltitude,
    PositionLocation, PositioningResponse, RadioType, WlanAccessPoint,
};
pub use routing::{
    ConsumptionModel, DriverParams, EvParams, FuelParams, HereNotice, HerePlace, HereRoute,
    HereRouteAction, HereRouteApiResponse, HereRouteOptions, HereRouteSection, HereRouteSummary,
    HereRouter, HereSpan, HereTurnAction, MaxSpeedOnSegment, ReturnField, RoutingMode,
    ScooterParams, TaxiParams, TollsParams, TrafficMode, TransportMode, TruckParams, Units,
};
pub use search::{
    AddressNamesMode, AutosuggestRequest, DiscoverRequest, DiscoverWithFeature, HereAddress,
    HereAutosuggestItem, HereAutosuggestOptions, HereAutosuggestResponse, HereCategory, HereChain,
    HereContact, HereDiscoverOptions, HereDiscoverResponse, HereEvStationFilter, HereEvSupplyType,
    HereFood, HereFuelStationFilter, HereGeocodeOptions, HereGeocoder, HereHighlightSection,
    HereHighlights, HereMapView, HereQueryTerm, HereRating, HereReference, HereRouteFilter,
    HereSearchItem, MobilityMode, PostalCodeMode, RankingMode, SearchType, ShowFeature,
    ShowMapReference, ShowNavAttribute, ShowRelated, ShowTranslation, WithFeature,
};
pub use tiling::{HereTileOptions, HereTileProvider, TileFormat, TileLayer};
pub use tour::{
    ActivityType, AsyncError, AsyncJobStatus, AsyncResource, AsyncStatus, AsyncSubmissionResult,
    CancellationStatus, Configuration, Fleet, FleetTraffic, Group, GroupPlacement, HealthResponse,
    HereTourOptions, HereTourPlanner, Job, JobPlace, JobPositionInTour, JobTask, JobTasks,
    MaxTimeOnVehicle, MultiObjective, Objective, Parking, ParkingPlace, Plan, PlanClustering,
    Profile, ProfileTraffic, Pudo, PudoAssignAt, PudoPlace, Relation, RelationType, Reload,
    RouteDetailType, RouteDetailsConfiguration, Shared, ShiftEnd, ShiftStart, SoftTimeWindow,
    Termination, TourActivity, TourLocation, TourNotice, TourProblem, TourSolution, TourStatistic,
    TourStop, TourStopTime, TourTimes, TourTour, UnassignedJob, UnassignedJobDetail,
    UnassignedJobReason, UnassignmentReasonCode, VehicleBreak, VehicleCosts, VehicleLimits,
    VehicleShift, VehicleType, VersionResponse,
};
pub use traffic::{
    AdvancedFeature, CriticalityLevel, HereAffectedItem, HereCurrentFlow, HereFlowItem,
    HereFlowOptions, HereFlowResponse, HereIncident, HereIncidentDescription, HereIncidentItem,
    HereIncidentsOptions, HereIncidentsResponse, HereLane, HereRoadInfo, HereSubSegment,
    HereTraffic, HereTrafficLocation, HereTrafficShapePoint, IncidentType, LocationReferencing,
    TrafficUnits,
};
