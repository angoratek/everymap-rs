use async_trait::async_trait;
use everymap_core::domains::traffic::{TrafficProvider, TrafficOptions, TrafficResponse};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions, PositioningResponse};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
use everymap_core::domains::geofencing::{GeofenceProvider, GeofenceOptions, GeofenceCreateOptions, GeofenceResponse, GeofenceResult};
use everymap_core::domains::tracking::{TripTracker, TripCreateOptions, TripUpdateOptions, TripResult};
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions, FraudResult};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::types::Coordinate;

/// Stub implementation of TrafficProvider for MapBox.
/// MapBox does not have a traffic API.
pub struct MapBoxTraffic;

#[async_trait]
impl TrafficProvider for MapBoxTraffic {
    async fn get_traffic(&self, _location: &Coordinate, _options: &TrafficOptions) -> EveryMapResult<TrafficResponse> {
        Err(EveryMapError::unsupported_domain("mapbox", "traffic"))
    }
}

/// Stub implementation of NetworkPositioner for MapBox.
/// MapBox does not have a positioning API.
pub struct MapBoxPositioner;

#[async_trait]
impl NetworkPositioner for MapBoxPositioner {
    async fn get_position(&self, _options: &PositioningOptions) -> EveryMapResult<PositioningResponse> {
        Err(EveryMapError::unsupported_domain("mapbox", "positioning"))
    }
}

/// Stub implementation of AttributeProvider for MapBox.
/// MapBox does not have an attributes API.
pub struct MapBoxAttributeProvider;

#[async_trait]
impl AttributeProvider for MapBoxAttributeProvider {
    async fn get_attributes(&self, _options: &AttributeOptions) -> EveryMapResult<AttributeResponse> {
        Err(EveryMapError::unsupported_domain("mapbox", "attributes"))
    }
}

/// Stub implementation of GeofenceProvider for MapBox.
/// MapBox does not have a geofencing API.
pub struct MapBoxGeofenceProvider;

#[async_trait]
impl GeofenceProvider for MapBoxGeofenceProvider {
    async fn search_geofences(&self, _options: &GeofenceOptions) -> EveryMapResult<GeofenceResponse> {
        Err(EveryMapError::unsupported_domain("mapbox", "geofencing"))
    }

    async fn create_geofence(&self, _options: &GeofenceCreateOptions) -> EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "geofencing"))
    }

    async fn get_geofence(&self, _id: &str) -> EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "geofencing"))
    }

    async fn delete_geofence(&self, _id: &str) -> EveryMapResult<()> {
        Err(EveryMapError::unsupported_domain("mapbox", "geofencing"))
    }
}

/// Stub implementation of TripTracker for MapBox.
/// MapBox does not have a trip tracking API.
pub struct MapBoxTripTracker;

#[async_trait]
impl TripTracker for MapBoxTripTracker {
    async fn create_trip(&self, _options: &TripCreateOptions) -> EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "tracking"))
    }

    async fn update_trip(&self, _options: &TripUpdateOptions) -> EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "tracking"))
    }

    async fn get_trip(&self, _trip_id: &str) -> EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "tracking"))
    }
}

/// Stub implementation of FraudDetector for MapBox.
/// MapBox does not have a fraud detection API.
pub struct MapBoxFraudDetector;

#[async_trait]
impl FraudDetector for MapBoxFraudDetector {
    async fn check_fraud(&self, _options: &FraudCheckOptions) -> EveryMapResult<FraudResult> {
        Err(EveryMapError::unsupported_domain("mapbox", "fraud"))
    }
}