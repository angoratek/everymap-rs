use everymap_core::domains::attributes::{AttributeOptions, AttributeProvider};
use everymap_core::domains::positioning::{NetworkPositioner, PositioningOptions};
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::types::Coordinate;
use everymap_providers_mapbox::{MapBoxAttributeProvider, MapBoxPositioner, MapBoxTraffic};

#[tokio::test]
async fn test_unsupported_traffic() {
    let provider = MapBoxTraffic;
    let coordinate = Coordinate::new(52.52, 13.405).unwrap();
    let options = TrafficOptions::default();
    let result = provider.get_traffic(&coordinate, &options).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("mapbox") && err.contains("traffic"));
}

#[tokio::test]
async fn test_unsupported_positioning() {
    let positioner = MapBoxPositioner;
    let options = PositioningOptions::default();
    let result = positioner.get_position(&options).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("mapbox") && err.contains("positioning"));
}

#[tokio::test]
async fn test_unsupported_attributes() {
    let provider = MapBoxAttributeProvider;
    let options = AttributeOptions::default();
    let result = provider.get_attributes(&options).await;
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("mapbox") && err.contains("attributes"));
}
