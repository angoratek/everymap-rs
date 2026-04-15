use async_trait::async_trait;
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions, PositioningResponse};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
use everymap_core::domains::geofencing::{GeofenceProvider, GeofenceOptions, GeofenceCreateOptions, GeofenceResponse, GeofenceResult};
use everymap_core::domains::tracking::{TripTracker, TripCreateOptions, TripUpdateOptions, TripResult};
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions, FraudResult};
use everymap_core::error::EveryMapError;

// Positioning — TomTom doesn't have a positioning API
pub struct TomTomPositioner;

#[async_trait]
impl NetworkPositioner for TomTomPositioner {
    async fn get_position(&self, _options: &PositioningOptions) -> everymap_core::error::EveryMapResult<PositioningResponse> {
        Err(EveryMapError::unsupported_domain("tomtom", "positioning"))
    }
}

// Attributes — TomTom doesn't have a dedicated road attributes API
pub struct TomTomAttributeProvider;

#[async_trait]
impl AttributeProvider for TomTomAttributeProvider {
    async fn get_attributes(&self, _options: &AttributeOptions) -> everymap_core::error::EveryMapResult<AttributeResponse> {
        Err(EveryMapError::unsupported_domain("tomtom", "attributes"))
    }
}

// Geofencing — TomTom doesn't have a geofencing API
pub struct TomTomGeofenceProvider;

#[async_trait]
impl GeofenceProvider for TomTomGeofenceProvider {
    async fn search_geofences(&self, _options: &GeofenceOptions) -> everymap_core::error::EveryMapResult<GeofenceResponse> {
        Err(EveryMapError::unsupported_domain("tomtom", "geofencing"))
    }

    async fn create_geofence(&self, _options: &GeofenceCreateOptions) -> everymap_core::error::EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "geofencing"))
    }

    async fn get_geofence(&self, _id: &str) -> everymap_core::error::EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "geofencing"))
    }

    async fn delete_geofence(&self, _id: &str) -> everymap_core::error::EveryMapResult<()> {
        Err(EveryMapError::unsupported_domain("tomtom", "geofencing"))
    }
}

// Trip Tracking — TomTom doesn't have a trip tracking API
pub struct TomTomTripTracker;

#[async_trait]
impl TripTracker for TomTomTripTracker {
    async fn create_trip(&self, _options: &TripCreateOptions) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "tracking"))
    }

    async fn update_trip(&self, _options: &TripUpdateOptions) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "tracking"))
    }

    async fn get_trip(&self, _trip_id: &str) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "tracking"))
    }
}

// Fraud Detection — TomTom doesn't have a fraud detection API
pub struct TomTomFraudDetector;

#[async_trait]
impl FraudDetector for TomTomFraudDetector {
    async fn check_fraud(&self, _options: &FraudCheckOptions) -> everymap_core::error::EveryMapResult<FraudResult> {
        Err(EveryMapError::unsupported_domain("tomtom", "fraud"))
    }
}