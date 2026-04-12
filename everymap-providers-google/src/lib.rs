pub mod client;
pub mod domain;

pub use client::GoogleClient;

// Shared types
pub use domain::geo::{GoogleLatLng, GoogleBounds};

// Search (Geocoding API)
pub use domain::search::{
    GoogleGeocoder,
    GoogleGeocodeResponse, GoogleGeocodeResult, GoogleGeometry,
    GoogleAddressComponent, GooglePlusCode,
};

// Routing (Directions API)
pub use domain::routing::{
    GoogleRouter,
    GoogleDirectionsResponse, GoogleRoute, GoogleRouteLeg, GoogleRouteStep,
    GoogleDistance, GoogleDuration, GooglePolyline, GoogleGeocodedWaypoint,
};

// Matching (Roads API snapToRoads)
pub use domain::matching::{
    GoogleRouteMatcher,
    GoogleSnapResponse, GoogleSnappedPoint, GoogleLocation, GoogleMatchOptions,
};

// Imaging (Static Maps API)
pub use domain::imaging::GoogleMapImageProvider;

// Positioning (Geolocation API)
pub use domain::positioning::{
    GooglePositioner,
    GooglePositioningOptions, GoogleWifiAccessPoint, GoogleCellTower,
    GoogleGeolocationResponse, GoogleGeolocationLocation,
};

// Attributes (Roads API speedLimits)
pub use domain::attributes::{
    GoogleAttributeProvider,
    GoogleAttributeOptions, GoogleSpeedLimitsResponse, GoogleSpeedLimit,
    GoogleSnappedSpeedPoint, GoogleSpeedLocation,
};

// Unsupported domains (return UnsupportedDomain error)
pub use domain::unsupported::{
    GoogleIsoline, GoogleTraffic, GoogleTourPlanner, GoogleTileProvider,
};