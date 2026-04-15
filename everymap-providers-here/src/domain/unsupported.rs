use async_trait::async_trait;
use everymap_core::domains::geofencing::{GeofenceProvider, GeofenceOptions, GeofenceCreateOptions, GeofenceResponse, GeofenceResult};
use everymap_core::domains::tracking::{TripTracker, TripCreateOptions, TripUpdateOptions, TripResult};
use everymap_core::domains::fraud::{FraudDetector, FraudCheckOptions, FraudResult};
use everymap_core::error::EveryMapError;

// Geofencing — HERE doesn't have a geofencing CRUD API
pub struct HereGeofenceProvider;

#[async_trait]
impl GeofenceProvider for HereGeofenceProvider {
    async fn search_geofences(&self, _options: &GeofenceOptions) -> everymap_core::error::EveryMapResult<GeofenceResponse> {
        Err(EveryMapError::unsupported_domain("here", "geofencing"))
    }

    async fn create_geofence(&self, _options: &GeofenceCreateOptions) -> everymap_core::error::EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("here", "geofencing"))
    }

    async fn get_geofence(&self, _id: &str) -> everymap_core::error::EveryMapResult<GeofenceResult> {
        Err(EveryMapError::unsupported_domain("here", "geofencing"))
    }

    async fn delete_geofence(&self, _id: &str) -> everymap_core::error::EveryMapResult<()> {
        Err(EveryMapError::unsupported_domain("here", "geofencing"))
    }
}

// Trip Tracking — HERE doesn't have a trip tracking API
pub struct HereTripTracker;

#[async_trait]
impl TripTracker for HereTripTracker {
    async fn create_trip(&self, _options: &TripCreateOptions) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("here", "tracking"))
    }

    async fn update_trip(&self, _options: &TripUpdateOptions) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("here", "tracking"))
    }

    async fn get_trip(&self, _trip_id: &str) -> everymap_core::error::EveryMapResult<TripResult> {
        Err(EveryMapError::unsupported_domain("here", "tracking"))
    }
}

// Fraud Detection — HERE doesn't have a fraud detection API
pub struct HereFraudDetector;

#[async_trait]
impl FraudDetector for HereFraudDetector {
    async fn check_fraud(&self, _options: &FraudCheckOptions) -> everymap_core::error::EveryMapResult<FraudResult> {
        Err(EveryMapError::unsupported_domain("here", "fraud"))
    }
}