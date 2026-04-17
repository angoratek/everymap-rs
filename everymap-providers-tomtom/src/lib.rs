pub mod client;
pub mod domain;
pub mod ext;

pub use client::TomTomClient;

// Extension traits
pub use ext::{TomTomGeocoderExt, TomTomTrafficExt};

// Shared types
pub use domain::geo::{TomTomLatLng, TomTomBounds};

// Search (Geocoding API)
pub use domain::search::{
    TomTomGeocoder,
    TomTomSearchResponse, TomTomSearchResult, TomTomAddress, TomTomPosition, TomTomBoundingBox,
    TomTomReverseGeocodeResult, TomTomReverseGeocodeAddress, TomTomReverseGeocodeBoundingBox,
};

// Routing (Routing API)
pub use domain::routing::{
    TomTomRouter,
    TomTomRouteResponse, TomTomRoute, TomTomRouteSummary, TomTomRouteLeg, TomTomRoutePoint,
};

// Traffic (Traffic API)
pub use domain::traffic::{
    TomTomTraffic,
    TomTomFlowResponse, TomTomFlowSegmentData, TomTomIncidentsResponse, TomTomIncident, TomTomIncidentGeometry,
};

// Isoline (Routing API — reachable range)
pub use domain::isoline::{
    TomTomIsoline,
    TomTomReachableRangeResponse, TomTomReachableRange,
};

// Matching (Snap to Roads API)
pub use domain::matching::{
    TomTomRouteMatcher,
    TomTomSnapResponse, TomTomProjectedPoint, TomTomProjectedGeometry, TomTomProjectedProperties,
    TomTomDistances,
};

// Tour (Waypoint Optimization API)
pub use domain::tour::{
    TomTomTourPlanner,
    TomTomOptimizationResponse, TomTomOptimizationSummary, TomTomTourRouteSummary,
};

// Tiling (Map Display API)
pub use domain::tiling::TomTomTileProvider;

// Imaging (Map Display API — static image)
pub use domain::imaging::TomTomMapImageProvider;

// Unsupported domains
pub use domain::unsupported::{TomTomPositioner, TomTomAttributeProvider};