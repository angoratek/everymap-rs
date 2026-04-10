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

// Unsupported domains (return UnsupportedDomain error)
pub use domain::unsupported::{
    GoogleIsoline, GoogleTraffic, GooglePositioner, GoogleRouteMatcher,
    GoogleTourPlanner, GoogleTileProvider, GoogleAttributeProvider, GoogleMapImageProvider,
};