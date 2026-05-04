use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_providers_tomtom::{TomTomAttributeProvider, TomTomPositioner};

#[tokio::test]
async fn test_unsupported_positioning() {
    let positioner = TomTomPositioner;
    let options = PositioningOptions::default();
    let result = positioner.get_position(&options).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("tomtom") && err.contains("positioning"));
}

#[tokio::test]
async fn test_unsupported_attributes() {
    let provider = TomTomAttributeProvider;
    let options = AttributeOptions::default();
    let result = provider.get_attributes(&options).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("tomtom") && err.contains("attributes"));
}
