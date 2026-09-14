//! Macros for generating unsupported domain stub implementations.
//!
//! These macros eliminate boilerplate when a provider doesn't support a particular
//! domain. Instead of manually writing trait implementations that return
//! `EveryMapError::unsupported_domain`, use the appropriate macro.

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
