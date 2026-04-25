use everymap_core::domains::isoline::{IsolineOptions, IsolineProvider};
use everymap_core::domains::tiling::{TileOptions, TileProvider};
use everymap_core::domains::tour::{TourOptions, TourPlanner};
use everymap_core::domains::traffic::{TrafficOptions, TrafficProvider};
use everymap_core::error::EveryMapError;
use everymap_core::types::Coordinate;
use everymap_providers_google::{
    GoogleIsoline, GoogleTileProvider, GoogleTourPlanner, GoogleTraffic,
};

#[tokio::test]
async fn test_unsupported_isoline() {
    let isoline = GoogleIsoline;
    let center = Coordinate::new(52.52, 13.405).unwrap();
    let result = isoline
        .get_isoline(&center, 1800.0, &IsolineOptions::default())
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "google");
            assert_eq!(domain, "isoline");
        }
        other => panic!("Expected UnsupportedDomain, got {:?}", other),
    }
}

#[tokio::test]
async fn test_unsupported_traffic() {
    let traffic = GoogleTraffic;
    let coord = Coordinate::new(52.52, 13.405).unwrap();
    let result = traffic
        .get_traffic(&coord, &TrafficOptions::default())
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "google");
            assert_eq!(domain, "traffic");
        }
        other => panic!("Expected UnsupportedDomain, got {:?}", other),
    }
}

#[tokio::test]
async fn test_unsupported_tour() {
    let tour = GoogleTourPlanner;
    let stops = vec![
        Coordinate::new(52.52, 13.405).unwrap(),
        Coordinate::new(48.856, 2.352).unwrap(),
    ];
    let result = tour.optimize_tour(&stops, &TourOptions::default()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "google");
            assert_eq!(domain, "tour");
        }
        other => panic!("Expected UnsupportedDomain, got {:?}", other),
    }
}

#[tokio::test]
async fn test_unsupported_tiling() {
    let tiling = GoogleTileProvider;
    let result = tiling
        .get_tile(14, 8802, 5373, &TileOptions::default())
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EveryMapError::UnsupportedDomain { provider, domain } => {
            assert_eq!(provider, "google");
            assert_eq!(domain, "tiling");
        }
        other => panic!("Expected UnsupportedDomain, got {:?}", other),
    }
}
