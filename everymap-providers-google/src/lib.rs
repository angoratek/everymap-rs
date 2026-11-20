pub mod client;
pub mod domain;
mod error_mapping;
pub mod ext;

use everymap_core::provider::{CredentialScheme, ProviderDescriptor, ServedContract, ServiceRoot};

/// The query parameter that carries the Google Maps API key.
pub const GOOGLE_API_KEY_QUERY_PARAMETER: &str = "key";

/// This crate's cross-domain provider description.
pub const PROVIDER_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
    key: error_mapping::GOOGLE_PROVIDER_NAME,
    display_name: "Google Maps",
    credential: CredentialScheme::QueryParameter(GOOGLE_API_KEY_QUERY_PARAMETER),
    root: ServiceRoot::PerDomain,
    attribution: None,
    terms: None,
    contracts: &[
        ServedContract::Geocoding,
        ServedContract::Routing,
        ServedContract::Matching,
        ServedContract::Positioning,
        ServedContract::Attributes,
        ServedContract::Imaging,
    ],
};

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
pub use domain::unsupported::{GoogleIsoline, GoogleTourPlanner, GoogleTraffic};

// Extension traits
pub use ext::{GoogleAttributeExt, GooglePositionerExt};

#[cfg(test)]
mod descriptor_tests {
    use super::{GOOGLE_API_KEY_QUERY_PARAMETER, PROVIDER_DESCRIPTOR};

    #[test]
    fn the_descriptor_reports_the_exported_query_parameter_name() {
        assert_eq!(
            PROVIDER_DESCRIPTOR.credential.query_parameter_name(),
            Some(GOOGLE_API_KEY_QUERY_PARAMETER)
        );
    }
}
