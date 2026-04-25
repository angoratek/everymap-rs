pub mod client;
pub mod domain;
pub mod ext;

pub use client::MapBoxClient;

// Extension traits
pub use ext::{MapBoxGeocoderExt, MapBoxRouterExt};

// Shared types
pub use domain::geo::{MapBoxBounds, MapBoxLatLng};

// Search (Geocoding API)
pub use domain::search::{
    MapBoxContext, MapBoxContextEntry, MapBoxCoordinates, MapBoxFeature, MapBoxGeocoder,
    MapBoxGeometry, MapBoxProperties, MapBoxSearchResponse,
};

// Routing (Directions API)
pub use domain::routing::{
    MapBoxManeuver, MapBoxRoute, MapBoxRouteLeg, MapBoxRouteResponse, MapBoxRouteStep,
    MapBoxRouter, MapBoxWaypoint,
};

// Isoline (Isochrone API)
pub use domain::isoline::{
    MapBoxIsochroneFeature, MapBoxIsochroneProperties, MapBoxIsochroneResponse, MapBoxIsoline,
};

// Matching (Map Matching API)
pub use domain::matching::{
    MapBoxMatchResponse, MapBoxMatching, MapBoxRouteMatcher, MapBoxTracepoint,
};

// Tour (Optimization API)
pub use domain::tour::{
    MapBoxOptWaypoint, MapBoxOptimizationResponse, MapBoxTourPlanner, MapBoxTrip,
};

// Tiling (Vector/Raster Tiles API)
pub use domain::tiling::MapBoxTileProvider;

// Imaging (Static Images API)
pub use domain::imaging::MapBoxMapImageProvider;

// Unsupported domains
pub use domain::unsupported::{MapBoxAttributeProvider, MapBoxPositioner, MapBoxTraffic};
