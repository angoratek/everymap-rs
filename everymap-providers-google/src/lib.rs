pub mod client;
pub mod domain;
pub mod ext;

pub use client::GoogleClient;

// Shared types
pub use domain::geo::{GoogleBounds, GoogleLatLng};

// Search (Geocoding API)
pub use domain::search::{
    GoogleAddressComponent, GoogleGeocodeResponse, GoogleGeocodeResult, GoogleGeocoder,
    GoogleGeometry, GooglePlusCode,
};

// Routing (Routes API v2)
pub use domain::routing::{
    GoogleNavigationInstruction, GoogleRoute, GoogleRouteLeg, GoogleRouteModifiers,
    GoogleRouteStep, GoogleRouter, GoogleRoutesLatLng, GoogleRoutesLocation, GoogleRoutesPolyline,
    GoogleRoutesRequest, GoogleRoutesResponse, GoogleTravelMode, GoogleViewport, GoogleWaypoint,
};

// Matching (Roads API snapToRoads)
pub use domain::matching::{
    GoogleLocation, GoogleMatchOptions, GoogleRouteMatcher, GoogleSnapResponse, GoogleSnappedPoint,
};

// Imaging (Static Maps API)
pub use domain::imaging::GoogleMapImageProvider;

// Positioning (Geolocation API)
pub use domain::positioning::{
    GoogleCellTower, GoogleGeolocationLocation, GoogleGeolocationResponse, GooglePositioner,
    GooglePositioningOptions, GoogleWifiAccessPoint,
};

// Attributes (Roads API speedLimits)
pub use domain::attributes::{
    GoogleAttributeOptions, GoogleAttributeProvider, GoogleSnappedSpeedPoint, GoogleSpeedLimit,
    GoogleSpeedLimitsResponse, GoogleSpeedLocation,
};

// Unsupported domains (return UnsupportedDomain error)
pub use domain::unsupported::{
    GoogleIsoline, GoogleTileProvider, GoogleTourPlanner, GoogleTraffic,
};

// Extension traits
pub use ext::{GoogleAttributeExt, GooglePositionerExt};
