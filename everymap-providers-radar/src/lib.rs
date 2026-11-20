pub mod client;
pub mod domain;
pub mod endpoints;
mod error_mapping;
pub mod ext;

use everymap_core::provider::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

/// This crate's cross-domain provider description.
pub const PROVIDER_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    key: error_mapping::RADAR_PROVIDER_NAME,
    display_name: "Radar",
    credential: CredentialScheme::Header("Authorization"),
    root: ServiceRoot::Single(endpoints::RADAR_API_ROOT),
    attribution: None,
    terms: None,
    contracts: &[
        ServedContract::Geocoding,
        ServedContract::Routing,
        ServedContract::Matching,
        ServedContract::Tour,
    ],
};

pub use client::RadarClient;

// Shared types
pub use domain::geo::RadarLatLng;
pub use domain::types::{
    RadarAvoid, RadarGeometry, RadarLocation, RadarMeta, RadarMetric, RadarTravelMode,
};

// Search (Geocoding API)
pub use domain::search::types::{
    RadarAddress, RadarAddressMetadata, RadarAddressValidationOptions,
    RadarAddressValidationResponse, RadarAutocompleteOptions, RadarAutocompleteResponse,
    RadarGeocodeResponse, RadarIpGeocodeResponse, RadarTimeZone, RadarValidatedAddress,
    RadarValidationResult,
};
pub use domain::search::RadarGeocoder;

// Routing (Directions API)
pub use domain::routing::types::{
    RadarDirectionsLeg, RadarDirectionsResponse, RadarDirectionsRoute, RadarDirectionsStep,
    RadarDistanceResponse, RadarDistanceRoutes, RadarMatrixEntry, RadarMatrixResponse,
    RadarModeDistance,
};
pub use domain::routing::RadarRouter;

// Matching (Route Match API)
pub use domain::matching::types::{RadarMatchedPoint, RadarRoadAttribute, RadarRouteMatchResponse};
pub use domain::matching::RadarRouteMatcher;

// Tour (Optimize Route API)
pub use domain::tour::types::{RadarOptimizeResponse, RadarOptimizedLeg, RadarOptimizedRoute};
pub use domain::tour::RadarTourPlanner;

// Unsupported domains (return UnsupportedDomain error)
pub use domain::unsupported::{
    RadarAttributeProvider, RadarIsoline, RadarMapImageProvider, RadarPositioner, RadarTraffic,
};

// Extension traits
pub use ext::{
    RadarChain, RadarGeocoderExt, RadarMatchingExt, RadarPlace, RadarPlaceSearchResponse,
    RadarRouterExt, RadarSearchExt,
};
