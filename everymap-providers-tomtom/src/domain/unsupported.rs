use async_trait::async_trait;
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions, PositioningResponse};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions, AttributeResponse};
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