//! Macros for generating unsupported domain stub implementations.
//!
//! These macros eliminate boilerplate when a provider doesn't support a particular
//! domain. Instead of manually writing trait implementations that return
//! `EveryMapError::unsupported_domain`, use the appropriate macro.
//!
//! # Example
//! ```ignore
//! // In everymap-providers-here/src/domain/unsupported.rs:
//! use everymap_core::unsupported::*;
//!
//! unsupported_geofence!(HereGeofenceProvider, "here");
//! unsupported_trip_tracker!(HereTripTracker, "here");
//! unsupported_fraud_detector!(HereFraudDetector, "here");
//! ```

/// Generates an unsupported `IsolineProvider` stub implementation.
#[macro_export]
macro_rules! unsupported_isoline {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::isoline::IsolineProvider for $struct_name {
            async fn get_isoline(
                &self,
                _center: &everymap_core::types::Coordinate,
                _range: f64,
                _options: &everymap_core::domains::isoline::IsolineOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::isoline::IsolineResponse,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "isoline",
                ))
            }
        }
    };
}

/// Generates an unsupported `TrafficProvider` stub implementation.
#[macro_export]
macro_rules! unsupported_traffic {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::traffic::TrafficProvider for $struct_name {
            async fn get_traffic(
                &self,
                _location: &everymap_core::types::Coordinate,
                _options: &everymap_core::domains::traffic::TrafficOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::traffic::TrafficResponse,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "traffic",
                ))
            }
        }
    };
}

/// Generates an unsupported `TourPlanner` stub implementation.
#[macro_export]
macro_rules! unsupported_tour {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::tour::TourPlanner for $struct_name {
            async fn optimize_tour(
                &self,
                _stops: &[everymap_core::types::Coordinate],
                _options: &everymap_core::domains::tour::TourOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::tour::TourResponse>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "tour",
                ))
            }
        }
    };
}

/// Generates an unsupported `TileProvider` stub implementation.
#[macro_export]
macro_rules! unsupported_tile {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::tiling::TileProvider for $struct_name {
            async fn get_tile(
                &self,
                _z: u32,
                _x: u32,
                _y: u32,
                _options: &everymap_core::domains::tiling::TileOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::tiling::TileResponse>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "tiling",
                ))
            }
        }
    };
}

/// Generates an unsupported `NetworkPositioner` stub implementation.
#[macro_export]
macro_rules! unsupported_positioner {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::positioning::NetworkPositioner for $struct_name {
            async fn get_position(
                &self,
                _options: &everymap_core::domains::positioning::PositioningOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::positioning::PositioningResponse,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "positioning",
                ))
            }
        }
    };
}

/// Generates an unsupported `AttributeProvider` stub implementation.
#[macro_export]
macro_rules! unsupported_attributes {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::attributes::AttributeProvider for $struct_name {
            async fn get_attributes(
                &self,
                _options: &everymap_core::domains::attributes::AttributeOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::attributes::AttributeResponse,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "attributes",
                ))
            }
        }
    };
}

/// Generates an unsupported `MapImageProvider` stub implementation.
#[macro_export]
macro_rules! unsupported_image {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::imaging::MapImageProvider for $struct_name {
            async fn get_image(
                &self,
                _center: &everymap_core::types::Coordinate,
                _zoom: u32,
                _size: (u32, u32),
                _options: &everymap_core::domains::imaging::ImageOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::imaging::ImageResponse> {
                Err(everymap_core::error::EveryMapError::unsupported_domain($provider, "imaging"))
            }
        }
    };
}

/// Generates an unsupported `GeofenceProvider` stub implementation (4 methods).
#[macro_export]
macro_rules! unsupported_geofence {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::geofencing::GeofenceProvider for $struct_name {
            async fn search_geofences(
                &self,
                _options: &everymap_core::domains::geofencing::GeofenceOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::geofencing::GeofenceResponse,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "geofencing",
                ))
            }

            async fn create_geofence(
                &self,
                _options: &everymap_core::domains::geofencing::GeofenceCreateOptions,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::geofencing::GeofenceResult,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "geofencing",
                ))
            }

            async fn get_geofence(
                &self,
                _id: &str,
            ) -> everymap_core::error::EveryMapResult<
                everymap_core::domains::geofencing::GeofenceResult,
            > {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "geofencing",
                ))
            }

            async fn delete_geofence(&self, _id: &str) -> everymap_core::error::EveryMapResult<()> {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider,
                    "geofencing",
                ))
            }
        }
    };
}

/// Generates an unsupported `TripTracker` stub implementation (3 methods).
#[macro_export]
macro_rules! unsupported_trip_tracker {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::tracking::TripTracker for $struct_name {
            async fn create_trip(
                &self,
                _options: &everymap_core::domains::tracking::TripCreateOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::tracking::TripResult>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "tracking",
                ))
            }

            async fn update_trip(
                &self,
                _options: &everymap_core::domains::tracking::TripUpdateOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::tracking::TripResult>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "tracking",
                ))
            }

            async fn get_trip(
                &self,
                _trip_id: &str,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::tracking::TripResult>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "tracking",
                ))
            }
        }
    };
}

/// Generates an unsupported `FraudDetector` stub implementation.
#[macro_export]
macro_rules! unsupported_fraud_detector {
    ($struct_name:ident, $provider:expr) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl everymap_core::domains::fraud::FraudDetector for $struct_name {
            async fn check_fraud(
                &self,
                _options: &everymap_core::domains::fraud::FraudCheckOptions,
            ) -> everymap_core::error::EveryMapResult<everymap_core::domains::fraud::FraudResult>
            {
                Err(everymap_core::error::EveryMapError::unsupported_domain(
                    $provider, "fraud",
                ))
            }
        }
    };
}
