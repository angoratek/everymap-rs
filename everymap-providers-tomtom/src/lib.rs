pub mod client;
pub mod domain;
mod error_mapping;
pub mod ext;

use everymap_core::provider::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

/// The query parameter that carries the TomTom API key.
pub const TOMTOM_API_KEY_QUERY_PARAMETER: &str = "key";

/// This crate's cross-domain provider description.
pub const PROVIDER_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    key: error_mapping::TOMTOM_PROVIDER_NAME,
    display_name: "TomTom",
    credential: CredentialScheme::QueryParameter(TOMTOM_API_KEY_QUERY_PARAMETER),
    root: ServiceRoot::PerDomain,
    attribution: None,
    terms: None,
    contracts: &[
        ServedContract::Geocoding,
        ServedContract::Routing,
        ServedContract::Traffic,
        ServedContract::RasterTiles,
        ServedContract::Isoline,
        ServedContract::Matching,
        ServedContract::Tour,
        ServedContract::Imaging,
    ],
};

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

#[cfg(test)]
mod descriptor_tests {
    use super::{PROVIDER_DESCRIPTOR, TOMTOM_API_KEY_QUERY_PARAMETER};

    #[test]
    fn the_descriptor_reports_the_exported_query_parameter_name() {
        assert_eq!(
            PROVIDER_DESCRIPTOR.credential.query_parameter_name(),
            Some(TOMTOM_API_KEY_QUERY_PARAMETER)
        );
    }
}
