pub mod client;
pub mod domain;
mod error_mapping;
pub mod ext;

use everymap_core::provider::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

/// This crate's cross-domain provider description.
pub const PROVIDER_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    key: error_mapping::GOOGLE_PROVIDER_NAME,
    display_name: "Google Maps",
    credential: CredentialScheme::QueryParameter("key"),
    root: ServiceRoot::PerDomain,
    attribution: None,
    terms: None,
    contracts: &[
        ServedContract::Geocoding,
        ServedContract::Routing,
        ServedContract::Elevation,
        ServedContract::Matching,
        ServedContract::Positioning,
        ServedContract::Attributes,
        ServedContract::Imaging,
    ],
};

pub use client::GoogleClient;

// Shared types
pub use domain::geo::{GoogleBounds, GoogleLatLng};

// Elevation (Elevation API)
pub use domain::elevation::{
    GoogleElevationProvider, GoogleElevationResponse, GoogleElevationResult,
    MAX_ELEVATION_LOCATIONS_PER_REQUEST,
};

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
pub use domain::unsupported::{GoogleIsoline, GoogleTourPlanner, GoogleTraffic};

// Extension traits
pub use ext::{GoogleAttributeExt, GooglePositionerExt};
