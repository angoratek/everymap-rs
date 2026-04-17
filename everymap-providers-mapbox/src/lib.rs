pub mod client;
pub mod domain;
pub mod ext;

pub use client::MapBoxClient;

// Extension traits
pub use ext::{MapBoxGeocoderExt, MapBoxRouterExt};

// Shared types
pub use domain::geo::{MapBoxLatLng, MapBoxBounds};

// Search (Geocoding API)
pub use domain::search::{
    MapBoxGeocoder,
    MapBoxSearchResponse, MapBoxFeature, MapBoxProperties, MapBoxCoordinates,
    MapBoxContext, MapBoxContextEntry, MapBoxGeometry,
};

// Routing (Directions API)
pub use domain::routing::{
    MapBoxRouter,
    MapBoxRouteResponse, MapBoxRoute, MapBoxRouteLeg, MapBoxRouteStep, MapBoxManeuver, MapBoxWaypoint,
};

// Isoline (Isochrone API)
pub use domain::isoline::{
    MapBoxIsoline,
    MapBoxIsochroneResponse, MapBoxIsochroneFeature, MapBoxIsochroneProperties,
};

// Matching (Map Matching API)
pub use domain::matching::{
    MapBoxRouteMatcher,
    MapBoxMatchResponse, MapBoxMatching, MapBoxTracepoint,
};

// Tour (Optimization API)
pub use domain::tour::{
    MapBoxTourPlanner,
    MapBoxOptimizationResponse, MapBoxTrip, MapBoxOptWaypoint,
};

// Tiling (Vector/Raster Tiles API)
pub use domain::tiling::MapBoxTileProvider;

// Imaging (Static Images API)
pub use domain::imaging::MapBoxMapImageProvider;

// Unsupported domains
pub use domain::unsupported::{MapBoxTraffic, MapBoxPositioner, MapBoxAttributeProvider};