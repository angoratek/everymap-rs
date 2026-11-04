//! Contract tests of the URL-template raster provider against a wiremock
//! server that replays a committed fixture.
//!
//! The harness asserts both directions: the request carries the expanded
//! template path, its query parameters and the declared headers, and the
//! typed tile echoes the requested format and resolution. Every request
//! targets the in-process server; no test reaches an external service.

use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::EveryMapError;
use everymap_providers_url_template_raster::{
    UrlTemplateRasterTileProvider, UrlTemplateRasterTileSource,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Header the tests configure and match on the wire.
const TEST_HEADER_NAME: &str = "x-api-version";

/// Value of the configured test header.
const TEST_HEADER_VALUE: &str = "2";

/// Reads the committed PNG fixture.
fn fixture_bytes() -> Vec<u8> {
    let fixture_path = format!(
        "{}/tests/fixtures/sample_tile.png",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(fixture_path).expect("the fixture must be readable")
}

/// A source whose template carries the optional `{s}` placeholder, a query
/// string and one declared request header.
fn source_with_subdomains(server: &MockServer) -> UrlTemplateRasterTileSource {
    UrlTemplateRasterTileSource {
        url_template: format!(
            "{}/tiles/{{s}}/{{z}}/{{x}}/{{y}}.png?style=base&zoom={{z}}",
            server.uri()
        ),
        source_name: "test/url-template-raster".to_string(),
        attribution_text: "Test imagery".to_string(),
        terms_text: "Test terms".to_string(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 15,
        image_format: RasterImageFormat::Png,
        tile_resolution: RasterTileResolution::Standard,
        subdomains: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        headers: vec![(TEST_HEADER_NAME.to_string(), TEST_HEADER_VALUE.to_string())],
        pacing: None,
    }
}

fn sample_request() -> RasterTileRequest {
    RasterTileRequest::new(
        2,
        1,
        3,
        RasterImageFormat::Jpeg,
        RasterTileResolution::Retina,
    )
}

#[tokio::test]
async fn a_fetch_expands_the_template_and_echoes_the_requested_format_and_resolution() {
    let server = MockServer::start().await;
    let fixture = fixture_bytes();
    Mock::given(method("GET"))
        .and(path("/tiles/a/3/2/1.png"))
        .and(query_param("style", "base"))
        .and(query_param("zoom", "3"))
        .and(header(TEST_HEADER_NAME, TEST_HEADER_VALUE))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(fixture.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let source = source_with_subdomains(&server);
    let provider =
        UrlTemplateRasterTileProvider::new(source).expect("the declared source is valid");

    let tile = provider
        .fetch_raster_tile(&sample_request())
        .await
        .expect("the recorded tile must decode");
    server.verify().await;

    assert_eq!(tile.tile_x, 2);
    assert_eq!(tile.tile_y, 1);
    assert_eq!(tile.zoom_level, 3);
    assert_eq!(tile.image_format, RasterImageFormat::Jpeg);
    assert_eq!(tile.tile_resolution, RasterTileResolution::Retina);
    assert_eq!(tile.image_bytes, fixture);
}

#[tokio::test]
async fn a_missing_tile_surfaces_the_http_status_without_retrying() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_string("tile not found"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = UrlTemplateRasterTileProvider::new(source_with_subdomains(&server))
        .expect("the declared source is valid");
    let error = provider
        .fetch_raster_tile(&sample_request())
        .await
        .expect_err("a missing tile must surface");
    server.verify().await;
    match error {
        EveryMapError::HttpError { status, .. } => assert_eq!(status, 404),
        other => panic!("expected an HTTP error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_rate_limited_response_surfaces_the_provider_and_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_string("slow down"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = UrlTemplateRasterTileProvider::new(source_with_subdomains(&server))
        .expect("the declared source is valid");
    let error = provider
        .fetch_raster_tile(&sample_request())
        .await
        .expect_err("a rate limit must surface");
    server.verify().await;
    match error {
        EveryMapError::RateLimited {
            provider,
            retry_after_secs,
        } => {
            assert_eq!(provider, "url-template-raster");
            assert_eq!(retry_after_secs, Some(60));
        }
        other => panic!("expected a rate-limit error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_malformed_template_is_refused() {
    let server = MockServer::start().await;
    for template in [
        "https://tiles.example.test/{x}/{y}.png",
        "https://tiles.example.test/{z}/{x}/{y}.png?q={q}",
        "https://{s}.tiles.example.test/{z}/{x}/{y}.png",
        "ftp://tiles.example.test/{z}/{x}/{y}.png",
    ] {
        let mut source = source_with_subdomains(&server);
        source.url_template = template.to_string();
        source.subdomains = Vec::new();
        let error = UrlTemplateRasterTileProvider::new(source)
            .err()
            .expect("a malformed template must be refused");
        assert!(
            matches!(error, EveryMapError::ValidationError(_)),
            "template {template:?} produced {error:?}"
        );
    }
}

#[tokio::test]
async fn a_malformed_descriptor_is_refused_verbatim() {
    let server = MockServer::start().await;
    let mut source = source_with_subdomains(&server);
    source.source_name = "test/inverted".to_string();
    source.minimum_zoom_level = 12;
    source.maximum_zoom_level = 8;
    let error = UrlTemplateRasterTileProvider::new(source)
        .err()
        .expect("an inverted zoom range must be refused");
    match error {
        EveryMapError::ValidationError(detail) => {
            assert!(detail.contains("test/inverted"), "{detail}");
            assert!(detail.contains("12..=8"), "{detail}");
        }
        other => panic!("expected a validation error, got {other:?}"),
    }

    let mut unnamed = source_with_subdomains(&server);
    unnamed.source_name = "   ".to_string();
    let error = UrlTemplateRasterTileProvider::new(unnamed)
        .err()
        .expect("a blank name must be refused");
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[test]
fn the_source_declaration_round_trips_through_json() {
    let source = UrlTemplateRasterTileSource {
        url_template: "https://{s}.tiles.example.test/{z}/{x}/{y}.png?style=base".to_string(),
        source_name: "test/url-template-raster".to_string(),
        attribution_text: "Test imagery".to_string(),
        terms_text: "Test terms".to_string(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 15,
        image_format: RasterImageFormat::Png,
        tile_resolution: RasterTileResolution::Standard,
        subdomains: vec!["a".to_string(), "b".to_string()],
        headers: vec![(TEST_HEADER_NAME.to_string(), TEST_HEADER_VALUE.to_string())],
        pacing: None,
    };
    let encoded = serde_json::to_string(&source).expect("the declaration must serialize");
    let decoded: UrlTemplateRasterTileSource =
        serde_json::from_str(&encoded).expect("the declaration must deserialize");
    assert_eq!(decoded, source);
}
