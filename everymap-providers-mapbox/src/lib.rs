pub mod client;
pub mod domain;
mod error_mapping;
pub mod ext;

use everymap_core::provider::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

/// This crate's cross-domain provider description.
pub const PROVIDER_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    key: error_mapping::MAPBOX_PROVIDER_NAME,
    display_name: "MapBox",
    credential: CredentialScheme::QueryParameter("access_token"),
    root: ServiceRoot::PerDomain,
    attribution: Some(domain::tiling::MAPBOX_VECTOR_ATTRIBUTION),
    terms: None,
    contracts: &[
        ServedContract::Geocoding,
        ServedContract::Routing,
        ServedContract::VectorTiles,
        ServedContract::Isoline,
        ServedContract::Matching,
        ServedContract::Tour,
        ServedContract::Imaging,
    ],
};

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
pub use domain::tiling::{
    MapBoxTileProvider, DEFAULT_MAPBOX_TILESET_ID, MAPBOX_VECTOR_ATTRIBUTION,
    MAPBOX_VECTOR_MAXIMUM_ZOOM_LEVEL,
};

// Imaging (Static Images API)
pub use domain::imaging::MapBoxMapImageProvider;

// Unsupported domains
pub use domain::unsupported::{MapBoxAttributeProvider, MapBoxPositioner, MapBoxTraffic};
