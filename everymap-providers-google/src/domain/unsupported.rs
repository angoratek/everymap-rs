use async_trait::async_trait;
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, IsolineResponse};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse};
use everymap_core::domains::tour::{TourPlanner, TourOptions, TourResponse};
use everymap_core::domains::tiling::{TileProvider, TileOptions, TileResponse};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;

// Isoline — Google doesn't have a dedicated isoline/reachability API
pub struct GoogleIsoline;

#[async_trait]
impl IsolineProvider for GoogleIsoline {
    async fn get_isoline(&self, _center: &Coordinate, _range: f64, _options: &IsolineOptions) -> everymap_core::error::EveryMapResult<IsolineResponse> {
        Err(EveryMapError::unsupported_domain("google", "isoline"))
    }
}

// Traffic — Google Maps doesn't have a traffic flow API (only traffic layers on map)
pub struct GoogleTraffic;

#[async_trait]
impl TrafficProvider for GoogleTraffic {
    async fn get_traffic(&self, _coordinate: &Coordinate, _options: &TrafficOptions) -> everymap_core::error::EveryMapResult<TrafficResponse> {
        Err(EveryMapError::unsupported_domain("google", "traffic"))
    }
}

// Tour — Google doesn't have a tour optimization API
pub struct GoogleTourPlanner;

#[async_trait]
impl TourPlanner for GoogleTourPlanner {
    async fn optimize_tour(&self, _stops: &[Coordinate], _options: &TourOptions) -> everymap_core::error::EveryMapResult<TourResponse> {
        Err(EveryMapError::unsupported_domain("google", "tour"))
    }
}

// Tiling — Google doesn't have a vector tile API
pub struct GoogleTileProvider;

#[async_trait]
impl TileProvider for GoogleTileProvider {
    async fn get_tile(&self, _z: u32, _x: u32, _y: u32, _options: &TileOptions) -> everymap_core::error::EveryMapResult<TileResponse> {
        Err(EveryMapError::unsupported_domain("google", "tiling"))
    }
}