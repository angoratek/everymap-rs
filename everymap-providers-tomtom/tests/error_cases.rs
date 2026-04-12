use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_core::domains::attributes::{AttributeProvider, AttributeOptions};
use everymap_core::error::EveryMapError;
use everymap_providers_tomtom::{TomTomPositioner, TomTomAttributeProvider};
use std::sync::Arc;

#[tokio::test]
async fn test_unsupported_positioning() {
    let positioner = TomTomPositioner;
    let opts = PositioningOptions::default();
    let result = positioner.get_position(&opts).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("tomtom") && err.contains("positioning"));
}

#[tokio::test]
async fn test_unsupported_attributes() {
    let provider = TomTomAttributeProvider;
    let opts = AttributeOptions::default();
    let result = provider.get_attributes(&opts).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("tomtom") && err.contains("attributes"));
}