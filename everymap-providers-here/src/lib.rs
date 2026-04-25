pub mod client;
pub mod domain;
pub mod ext;
pub mod util;

pub use client::HereClient;
pub use domain::attributes::{
    AttributeFormat, AttributeLayer, DirectionalSpeedLimit, HereAdminAreaAttributeFeature,
    HereAdminAreaFeature, HereAdminAreasResponse, HereAttributeOptions, HereAttributeProvider,
    HereBuildingAttributeFeature, HereBuildingFeature, HereBuildingsResponse,
    HereLandmarkAttributeFeature, HereLandmarkFeature, HereLandmarksResponse,
    HereRoadAttributeFeature, HereRoadAttributesResponse, HereRoadFeature,
    HereSegmentAttributeFeature, HereSegmentAttributesResponse, HereSegmentFeature, LocalizedText,
};
pub use domain::geo::HereLatLng;
pub use domain::imaging::{HereImageOptions, HereMapImageProvider, ImageFormat, MapStyle};
pub use domain::isoline::{HereIsoline, HereIsolineOptions};
pub use domain::matching::{
    AvoidFeature, EmissionType, HazardousGoodsType, HereMatchApiResponse, HereMatchError,
    HereMatchSummary, HereMatchedLeg, HereMatchedLink, HereMatchedManeuver, HereMatchedPoint,
    HereMatchedRoute, HereMatchingOptions, HereRouteMatcher, InstructionFormat, LegalConstraint,
    MatchFuelType, MatchMode, TrailerType, TunnelCategory,
};
pub use domain::positioning::{
    BluetoothBeacon, CellTower, Fallback, HerePositioner, HerePositioningOptions, PositionAltitude,
    PositionLocation, PositioningResponse, RadioType, WlanAccessPoint,
};
pub use domain::routing::{
    ConsumptionModel, DriverParams, EvParams, FuelParams, HereNotice, HerePlace, HereRoute,
    HereRouteAction, HereRouteApiResponse, HereRouteOptions, HereRouteSection, HereRouteSummary,
    HereRouter, HereSpan, HereTurnAction, MaxSpeedOnSegment, ReturnField, RoutingMode,
    ScooterParams, TaxiParams, TollsParams, TrafficMode, TransportMode, TruckParams, Units,
};
pub use domain::search::{
    AddressNamesMode, DiscoverWithFeature, HereAddress, HereAutosuggestItem,
    HereAutosuggestOptions, HereAutosuggestResponse, HereCategory, HereChain, HereContact,
    HereDiscoverOptions, HereDiscoverResponse, HereEvStationFilter, HereEvSupplyType, HereFood,
    HereFuelStationFilter, HereGeocodeOptions, HereGeocoder, HereHighlightSection, HereHighlights,
    HereMapView, HereQueryTerm, HereRating, HereReference, HereRouteFilter, HereSearchItem,
    MobilityMode, PostalCodeMode, RankingMode, SearchType, ShowFeature, ShowMapReference,
    ShowNavAttribute, ShowRelated, ShowTranslation, WithFeature,
};
pub use domain::tiling::{HereTileOptions, HereTileProvider, TileFormat, TileLayer};
pub use domain::tour::{
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
pub use domain::traffic::HereTraffic;

// Extension traits for HERE-specific capabilities
pub use ext::{
    HereAttributeExt, HereGeocoderExt, HerePositionerExt, HereTourPlannerExt, HereTrafficExt,
};
