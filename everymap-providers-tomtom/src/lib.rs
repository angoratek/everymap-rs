pub mod client;
pub mod domain;
pub mod ext;

pub use client::TomTomClient;

// Extension traits
pub use ext::{TomTomGeocoderExt, TomTomTrafficExt};

// Shared types
pub use domain::geo::{TomTomBounds, TomTomLatLng};

// Search (Geocoding API)
pub use domain::search::{
    TomTomAddress, TomTomBoundingBox, TomTomGeocoder, TomTomPosition, TomTomReverseGeocodeAddress,
    TomTomReverseGeocodeBoundingBox, TomTomReverseGeocodeResult, TomTomSearchResponse,
    TomTomSearchResult,
};

// Routing (Routing API)
pub use domain::routing::{
    TomTomRoute, TomTomRouteLeg, TomTomRoutePoint, TomTomRouteResponse, TomTomRouteSummary,
    TomTomRouter,
};

// Traffic (Traffic API)
pub use domain::traffic::{
    TomTomFlowResponse, TomTomFlowSegmentData, TomTomIncident, TomTomIncidentGeometry,
    TomTomIncidentsResponse, TomTomTraffic,
};

// Isoline (Routing API — reachable range)
pub use domain::isoline::{TomTomIsoline, TomTomReachableRange, TomTomReachableRangeResponse};

// Matching (Snap to Roads API)
pub use domain::matching::{
    TomTomDistances, TomTomProjectedGeometry, TomTomProjectedPoint, TomTomProjectedProperties,
    TomTomRouteMatcher, TomTomSnapResponse,
};

// Tour (Waypoint Optimization API)
pub use domain::tour::{
    TomTomOptimizationResponse, TomTomOptimizationSummary, TomTomTourPlanner,
    TomTomTourRouteSummary,
};

// Tiling (Map Display API)
pub use domain::tiling::TomTomTileProvider;

// Imaging (Map Display API — static image)
pub use domain::imaging::TomTomMapImageProvider;

// Unsupported domains
pub use domain::unsupported::{TomTomAttributeProvider, TomTomPositioner};
