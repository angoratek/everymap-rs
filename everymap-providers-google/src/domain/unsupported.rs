use async_trait::async_trait;
use everymap_core::domains::isoline::{IsolineProvider, IsolineOptions, IsolineResponse};
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions, PositioningResponse};
use everymap_core::domains::matching::{RouteMatcher, MatchingOptions, TraceResponse};
use everymap_core::domains::tour::{TourPlanner, TourOptions, TourResponse};
use everymap_core::domains::tiling::{TileProvider, TileOptions, TileResponse};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
use everymap_core::domains::imaging::{MapImageProvider, ImageOptions, ImageResponse};
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

// Positioning — Google Geolocation API exists but is separate and paid
pub struct GooglePositioner;

#[async_trait]
impl NetworkPositioner for GooglePositioner {
    async fn get_position(&self, _options: &PositioningOptions) -> everymap_core::error::EveryMapResult<PositioningResponse> {
        Err(EveryMapError::unsupported_domain("google", "positioning"))
    }
}

// Matching — Google Roads API (snapToRoads) exists but is limited
pub struct GoogleRouteMatcher;

#[async_trait]
impl RouteMatcher for GoogleRouteMatcher {
    async fn match_route(&self, _points: &[Coordinate], _options: &MatchingOptions) -> everymap_core::error::EveryMapResult<TraceResponse> {
        Err(EveryMapError::unsupported_domain("google", "matching"))
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

// Attributes — Google doesn't have a road attributes API
pub struct GoogleAttributeProvider;

#[async_trait]
impl AttributeProvider for GoogleAttributeProvider {
    async fn get_attributes(&self, _options: &AttributeOptions) -> everymap_core::error::EveryMapResult<AttributeResponse> {
        Err(EveryMapError::unsupported_domain("google", "attributes"))
    }
}

// Imaging — Google Static Maps API (partial support, future)
pub struct GoogleMapImageProvider;

#[async_trait]
impl MapImageProvider for GoogleMapImageProvider {
    async fn get_image(&self, _center: &Coordinate, _zoom: u32, _size: (u32, u32), _options: &ImageOptions) -> everymap_core::error::EveryMapResult<ImageResponse> {
        Err(EveryMapError::unsupported_domain("google", "imaging"))
    }
}