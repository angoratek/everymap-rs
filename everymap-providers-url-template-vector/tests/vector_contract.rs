//! Contract tests of the URL-template vector provider against a wiremock
//! server that replays committed fixtures.
//!
//! The harness asserts both directions: the request carries the expanded
//! template path, its query parameters and the declared headers, and each
//! response channel maps to the contract's typed tile or error. Every request
//! targets the in-process server; no test reaches an external service.

use everymap_core::domains::tiling::{
    VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::EveryMapError;
use everymap_providers_url_template_vector::{
    UrlTemplateVectorTileProvider, UrlTemplateVectorTileSource,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Header the tests configure and match on the wire.
const TEST_HEADER_NAME: &str = "x-api-version";

/// Value of the configured test header.
const TEST_HEADER_VALUE: &str = "2";

/// Reads one committed fixture.
fn fixture_bytes(name: &str) -> Vec<u8> {
    let fixture_path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(fixture_path).expect("the fixture must be readable")
}

fn source_descriptor(tile_encoding: VectorTileEncoding) -> VectorTileSourceDescriptor {
    VectorTileSourceDescriptor {
        source_name: "test/url-template-vector".to_string(),
        attribution_text: "Test tiles".to_string(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 15,
        tile_encoding,
        style_hints: vec!["openmaptiles".to_string()],
    }
}

/// A source whose template carries the optional `{s}` placeholder, a query
/// string and one declared request header.
fn source_with_subdomains(server: &MockServer) -> UrlTemplateVectorTileSource {
    UrlTemplateVectorTileSource {
        url_template: format!(
            "{}/tiles/{{s}}/{{z}}/{{x}}/{{y}}.mvt?style=base&zoom={{z}}",
            server.uri()
        ),
        source_descriptor: source_descriptor(VectorTileEncoding::GzippedMapboxVectorTile),
        terms_text: "Test terms".to_string(),
        subdomains: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        headers: vec![(TEST_HEADER_NAME.to_string(), TEST_HEADER_VALUE.to_string())],
        pacing: None,
    }
}

#[tokio::test]
async fn a_fetch_expands_the_template_and_reports_the_gzip_encoding() {
    let server = MockServer::start().await;
    let fixture = fixture_bytes("sample_tile.mvt.gz");
    Mock::given(method("GET"))
        .and(path("/tiles/a/3/2/1.mvt"))
        .and(query_param("style", "base"))
        .and(query_param("zoom", "3"))
        .and(header(TEST_HEADER_NAME, TEST_HEADER_VALUE))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(fixture.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let source = source_with_subdomains(&server);
    let descriptor = source.source_descriptor.clone();
    let provider =
        UrlTemplateVectorTileProvider::new(source).expect("the declared source is valid");

    let tile = provider
        .fetch_vector_tile(2, 1, 3)
        .await
        .expect("the recorded tile must decode");
    server.verify().await;

    assert_eq!(tile.tile_x, 2);
    assert_eq!(tile.tile_y, 1);
    assert_eq!(tile.zoom_level, 3);
    assert_eq!(
        tile.tile_encoding,
        VectorTileEncoding::GzippedMapboxVectorTile
    );
    assert_eq!(tile.tile_bytes, fixture);
    assert_eq!(provider.source_descriptor(), descriptor);
    assert_eq!(
        provider.source_descriptor().style_hints,
        vec!["openmaptiles"]
    );
}

#[tokio::test]
async fn plain_mvt_bytes_report_the_uncompressed_encoding() {
    let server = MockServer::start().await;
    let fixture = fixture_bytes("sample_tile.mvt");
    Mock::given(method("GET"))
        .and(path("/tiles/3/2/1.mvt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(fixture.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
        url_template: format!("{}/tiles/{{z}}/{{x}}/{{y}}.mvt", server.uri()),
        source_descriptor: source_descriptor(VectorTileEncoding::MapboxVectorTile),
        ..source_with_subdomains(&server)
    })
    .expect("the declared source is valid");

    let tile = provider
        .fetch_vector_tile(2, 1, 3)
        .await
        .expect("the recorded tile must decode");
    server.verify().await;
    assert_eq!(tile.tile_encoding, VectorTileEncoding::MapboxVectorTile);
    assert_eq!(tile.tile_bytes, fixture);
}

#[tokio::test]
async fn an_empty_tile_body_is_a_validation_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tiles/3/2/1.mvt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(Vec::new()))
        .expect(1)
        .mount(&server)
        .await;

    let provider = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
        url_template: format!("{}/tiles/{{z}}/{{x}}/{{y}}.mvt", server.uri()),
        ..source_with_subdomains(&server)
    })
    .expect("the declared source is valid");

    let error = provider
        .fetch_vector_tile(2, 1, 3)
        .await
        .expect_err("an empty payload must surface");
    server.verify().await;
    assert!(matches!(error, EveryMapError::ValidationError(_)));
}

#[tokio::test]
async fn a_missing_tile_surfaces_the_http_status_without_retrying() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_string("tile not found"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = UrlTemplateVectorTileProvider::new(source_with_subdomains(&server))
        .expect("the declared source is valid");
    let error = provider
        .fetch_vector_tile(2, 1, 3)
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

    let provider = UrlTemplateVectorTileProvider::new(source_with_subdomains(&server))
        .expect("the declared source is valid");
    let error = provider
        .fetch_vector_tile(2, 1, 3)
        .await
        .expect_err("a rate limit must surface");
    server.verify().await;
    match error {
        EveryMapError::RateLimited {
            provider,
            retry_after_secs,
        } => {
            assert_eq!(provider, "url-template-vector");
            assert_eq!(retry_after_secs, Some(60));
        }
        other => panic!("expected a rate-limit error, got {other:?}"),
    }
}

#[test]
fn a_malformed_template_is_refused() {
    for template in [
        "https://tiles.example.test/{x}/{y}.mvt",
        "https://tiles.example.test/{z}/{x}/{y}.mvt?q={q}",
        "https://{s}.tiles.example.test/{z}/{x}/{y}.mvt",
        "ftp://tiles.example.test/{z}/{x}/{y}.mvt",
    ] {
        let error = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
            url_template: template.to_string(),
            source_descriptor: source_descriptor(VectorTileEncoding::MapboxVectorTile),
            terms_text: String::new(),
            subdomains: Vec::new(),
            headers: Vec::new(),
            pacing: None,
        })
        .err()
        .expect("a malformed template must be refused");
        assert!(
            matches!(error, EveryMapError::ValidationError(_)),
            "template {template:?} produced {error:?}"
        );
    }
}

#[test]
fn a_malformed_descriptor_is_refused_verbatim() {
    let inverted = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
        url_template: "https://tiles.example.test/{z}/{x}/{y}.mvt".to_string(),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "test/inverted".to_string(),
            minimum_zoom_level: 14,
            maximum_zoom_level: 6,
            ..source_descriptor(VectorTileEncoding::MapboxVectorTile)
        },
        terms_text: String::new(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    })
    .err()
    .expect("an inverted zoom range must be refused");
    match inverted {
        EveryMapError::ValidationError(detail) => {
            assert!(detail.contains("test/inverted"), "{detail}");
            assert!(detail.contains("14..=6"), "{detail}");
        }
        other => panic!("expected a validation error, got {other:?}"),
    }

    let unnamed = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
        url_template: "https://tiles.example.test/{z}/{x}/{y}.mvt".to_string(),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "   ".to_string(),
            ..source_descriptor(VectorTileEncoding::MapboxVectorTile)
        },
        terms_text: String::new(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    })
    .err()
    .expect("a blank source name must be refused");
    assert!(matches!(unnamed, EveryMapError::ValidationError(_)));
}

#[test]
fn the_source_declaration_round_trips_through_json() {
    let source = UrlTemplateVectorTileSource {
        url_template: "https://{s}.tiles.example.test/{z}/{x}/{y}.mvt?style=base".to_string(),
        source_descriptor: source_descriptor(VectorTileEncoding::GzippedMapboxVectorTile),
        terms_text: "Test terms".to_string(),
        subdomains: vec!["a".to_string(), "b".to_string()],
        headers: vec![(TEST_HEADER_NAME.to_string(), TEST_HEADER_VALUE.to_string())],
        pacing: None,
    };
    let encoded = serde_json::to_string(&source).expect("the declaration must serialize");
    let decoded: UrlTemplateVectorTileSource =
        serde_json::from_str(&encoded).expect("the declaration must deserialize");
    assert_eq!(decoded, source);
}
