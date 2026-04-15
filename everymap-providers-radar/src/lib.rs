pub mod client;
pub mod domain;
pub mod ext;

pub use client::RadarClient;

// Shared types
pub use domain::types::{RadarMeta, RadarMetric, RadarLocation, RadarGeometry, RadarTravelMode, RadarAvoid};
pub use domain::geo::RadarLatLng;

// Search (Geocoding API)
pub use domain::search::{
    RadarGeocoder,
};
pub use domain::search::types::{
    RadarGeocodeResponse, RadarAddress, RadarTimeZone,
    RadarIpGeocodeResponse, RadarAutocompleteResponse, RadarAutocompleteOptions,
    RadarAddressValidationResponse, RadarValidatedAddress, RadarValidationResult,
    RadarAddressValidationOptions, RadarAddressMetadata,
};

// Routing (Directions API)
pub use domain::routing::{
    RadarRouter,
};
pub use domain::routing::types::{
    RadarDirectionsResponse, RadarDirectionsRoute, RadarDirectionsLeg,
    RadarDirectionsStep, RadarDistanceResponse, RadarDistanceRoutes,
    RadarModeDistance, RadarMatrixResponse, RadarMatrixEntry,
};

// Matching (Route Match API)
pub use domain::matching::{
    RadarRouteMatcher,
};
pub use domain::matching::types::{
    RadarRouteMatchResponse, RadarMatchedPoint, RadarRoadAttribute,
};

// Tour (Optimize Route API)
pub use domain::tour::{
    RadarTourPlanner,
};
pub use domain::tour::types::{
    RadarOptimizeResponse, RadarOptimizedRoute, RadarOptimizedLeg,
};

// Geofencing
pub use domain::geofencing::{
    RadarGeofenceProvider,
};
pub use domain::geofencing::types::{
    RadarGeofence, RadarGeofenceSearchResponse, RadarGeofenceCreateResponse,
    RadarGeofenceGetResponse,
};

// Tracking
pub use domain::tracking::{
    RadarTripTracker,
};
pub use domain::tracking::types::{
    RadarTrip, RadarTripCreateResponse, RadarTripGetResponse,
};

// Fraud
pub use domain::fraud::{
    RadarFraudDetector,
};
pub use domain::fraud::types::{
    RadarTrackResponse, RadarTrackUser, RadarFraudData,
};

// Unsupported domains (return UnsupportedDomain error)
pub use domain::unsupported::{
    RadarIsoline, RadarTraffic, RadarTileProvider, RadarPositioner,
    RadarAttributeProvider, RadarMapImageProvider,
};

// Extension traits
pub use ext::{
    RadarGeocoderExt, RadarRouterExt, RadarSearchExt, RadarMatchingExt,
    RadarPlaceSearchResponse, RadarPlace, RadarChain,
};