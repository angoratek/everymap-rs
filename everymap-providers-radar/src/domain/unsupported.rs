use async_trait::async_trait;
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, IsolineResponse};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse};
use everymap_core::domains::tiling::{TileProvider, TileOptions, TileResponse};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions, PositioningResponse};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
use everymap_core::domains::imaging::{MapImageProvider, ImageOptions, ImageResponse};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;

pub struct RadarIsoline;

#[async_trait]
impl IsolineProvider for RadarIsoline {
    async fn get_isoline(&self, _: &Coordinate, _: f64, _: &IsolineOptions) -> EveryMapResult<IsolineResponse> {
        Err(EveryMapError::unsupported_domain("radar", "isoline"))
    }
}

pub struct RadarTraffic;

#[async_trait]
impl TrafficProvider for RadarTraffic {
    async fn get_traffic(&self, _: &Coordinate, _: &TrafficOptions) -> EveryMapResult<TrafficResponse> {
        Err(EveryMapError::unsupported_domain("radar", "traffic"))
    }
}

pub struct RadarTileProvider;

#[async_trait]
impl TileProvider for RadarTileProvider {
    async fn get_tile(&self, _: u32, _: u32, _: u32, _: &TileOptions) -> EveryMapResult<TileResponse> {
        Err(EveryMapError::unsupported_domain("radar", "tiling"))
    }
}

pub struct RadarPositioner;

#[async_trait]
impl NetworkPositioner for RadarPositioner {
    async fn get_position(&self, _: &PositioningOptions) -> EveryMapResult<PositioningResponse> {
        Err(EveryMapError::unsupported_domain("radar", "positioning"))
    }
}

pub struct RadarAttributeProvider;

#[async_trait]
impl AttributeProvider for RadarAttributeProvider {
    async fn get_attributes(&self, _: &AttributeOptions) -> EveryMapResult<AttributeResponse> {
        Err(EveryMapError::unsupported_domain("radar", "attributes"))
    }
}

pub struct RadarMapImageProvider;

#[async_trait]
impl MapImageProvider for RadarMapImageProvider {
    async fn get_image(&self, _: &Coordinate, _: u32, _: (u32, u32), _: &ImageOptions) -> EveryMapResult<ImageResponse> {
        Err(EveryMapError::unsupported_domain("radar", "imaging"))
    }
}