//! Contract tests of the MapQuest raster tile provider against a committed
//! PNG fixture.
//!
//! The tests assert the documented path segmentation for both resolutions and
//! formats, that the returned tile echoes the request, and that coordinates
//! outside the service's bounds are rejected before any request.

mod common;

use common::{client_for, fixture_bytes, TEST_API_KEY, TEST_USER_AGENT};
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::EveryMapError;
use everymap_providers_mapquest::MapQuestRasterTileProvider;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_standard_png_tile_is_fetched_and_echoed() {
    let server = MockServer::start().await;
    let tile_bytes = fixture_bytes("raster_tile.png");
    Mock::given(method("GET"))
        .and(path("/tiles/v1/render/latest/vivid/13/1873/3374/256/png"))
        .and(query_param("key", TEST_API_KEY))
        .and(header("user-agent", TEST_USER_AGENT))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(tile_bytes.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestRasterTileProvider::new(client_for(&server));
    let request = RasterTileRequest::new(
        1873,
        3374,
        13,
        RasterImageFormat::Png,
        RasterTileResolution::Standard,
    );
    let tile = provider
        .fetch_raster_tile(&request)
        .await
        .expect("the recorded tile must decode");
    server.verify().await;

    assert_eq!(tile.tile_x, 1873);
    assert_eq!(tile.tile_y, 3374);
    assert_eq!(tile.zoom_level, 13);
    assert_eq!(tile.image_format, RasterImageFormat::Png);
    assert_eq!(tile.tile_resolution, RasterTileResolution::Standard);
    assert_eq!(tile.image_bytes, tile_bytes);
}

#[tokio::test]
async fn a_retina_jpeg_tile_uses_the_double_resolution_segment() {
    let server = MockServer::start().await;
    let tile_bytes = fixture_bytes("raster_tile.png");
    Mock::given(method("GET"))
        .and(path(
            "/tiles/v1/render/latest/vivid/13/1873/3374/256@2x/jpeg",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(tile_bytes.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = MapQuestRasterTileProvider::new(client_for(&server));
    let request = RasterTileRequest::new(
        1873,
        3374,
        13,
        RasterImageFormat::Jpeg,
        RasterTileResolution::Retina,
    );
    let tile = provider
        .fetch_raster_tile(&request)
        .await
        .expect("the recorded tile must decode");
    server.verify().await;

    assert_eq!(tile.image_format, RasterImageFormat::Jpeg);
    assert_eq!(tile.tile_resolution, RasterTileResolution::Retina);
    assert_eq!(tile.image_bytes, tile_bytes);
}

#[tokio::test]
async fn an_explicit_tile_style_is_used_verbatim() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tiles/v1/render/latest/satellite/5/4/11/256/png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(fixture_bytes("raster_tile.png")))
        .expect(1)
        .mount(&server)
        .await;

    let provider =
        MapQuestRasterTileProvider::with_tile_style(client_for(&server), "satellite".to_string());
    assert_eq!(provider.tile_style(), "satellite");
    let request = RasterTileRequest::new(
        4,
        11,
        5,
        RasterImageFormat::Png,
        RasterTileResolution::Standard,
    );
    provider
        .fetch_raster_tile(&request)
        .await
        .expect("the recorded tile must decode");
    server.verify().await;
}

#[tokio::test]
async fn out_of_range_coordinates_are_rejected_without_a_request() {
    let server = MockServer::start().await;
    let provider = MapQuestRasterTileProvider::new(client_for(&server));

    let too_deep = RasterTileRequest::new(
        0,
        0,
        21,
        RasterImageFormat::Png,
        RasterTileResolution::Standard,
    );
    assert!(matches!(
        provider.fetch_raster_tile(&too_deep).await,
        Err(EveryMapError::ValidationError(_))
    ));

    let out_of_range = RasterTileRequest::new(
        8192,
        0,
        13,
        RasterImageFormat::Png,
        RasterTileResolution::Standard,
    );
    assert!(matches!(
        provider.fetch_raster_tile(&out_of_range).await,
        Err(EveryMapError::ValidationError(_))
    ));
}
