use everymap_core::domains::attributes::AttributeProvider;
use everymap_core::domains::imaging::MapImageProvider;
use everymap_core::domains::isoline::IsolineProvider;
use everymap_core::domains::positioning::NetworkPositioner;
use everymap_core::domains::tiling::TileProvider;
use everymap_core::domains::traffic::TrafficProvider;
use everymap_core::error::EveryMapError;
use everymap_providers_radar::*;

#[tokio::test]
async fn test_unsupported_isoline() {
    let result = RadarIsoline
        .get_isoline(
            &everymap_core::types::Coordinate::ORIGIN,
            1000.0,
            &everymap_core::domains::isoline::IsolineOptions::default(),
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "radar");
            assert_eq!(domain, "isoline");
        }
        _ => panic!("Expected UnsupportedDomain error, got {:?}", err),
    }
}

#[tokio::test]
async fn test_unsupported_traffic() {
    let result = RadarTraffic
        .get_traffic(
            &everymap_core::types::Coordinate::ORIGIN,
            &everymap_core::domains::traffic::TrafficOptions::default(),
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "radar");
            assert_eq!(domain, "traffic");
        }
        _ => panic!("Expected UnsupportedDomain error, got {:?}", err),
    }
}

#[tokio::test]
async fn test_unsupported_tiling() {
    let result = RadarTileProvider
        .get_tile(
            10,
            5,
            3,
            &everymap_core::domains::tiling::TileOptions::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unsupported_positioning() {
    let result = RadarPositioner
        .get_position(&everymap_core::domains::positioning::PositioningOptions::default())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unsupported_attributes() {
    let result = RadarAttributeProvider
        .get_attributes(&everymap_core::domains::attributes::AttributeOptions::default())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unsupported_imaging() {
    let result = RadarMapImageProvider
        .get_image(
            &everymap_core::types::Coordinate::ORIGIN,
            10,
            (600, 400),
            &everymap_core::domains::imaging::ImageOptions::default(),
        )
        .await;
    assert!(result.is_err());
}
