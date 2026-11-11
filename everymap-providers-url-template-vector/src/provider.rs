//! The provider that serves one URL-template vector tileset.
//!
//! One instance serves one tileset: the validated template turns tile
//! coordinates into request URLs, the source declaration names the tileset for
//! caching and attribution, and every fetch is paced by the source's declared
//! token bucket and retried in bounds through the shared client.

use crate::client::UrlTemplateVectorClient;
use crate::pacing::TileSourcePacing;
use crate::url_template::UrlTemplate;
use async_trait::async_trait;
use everymap_core::client::TokenBucket;
use everymap_core::domains::tiling::{
    VectorTileData, VectorTileProvider, VectorTileSourceDescriptor,
};
use everymap_core::error::EveryMapResult;
use everymap_core::http::{HttpTransport, Method};
use everymap_core::tile_source::{validate_tile_coordinates, validate_tile_source_declaration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// What one URL-template vector source serves.
///
/// The source descriptor names the tileset for caching, attribution and
/// rendering, while the remaining fields describe how requests are built: the
/// template, the values substituted for its optional `{s}` placeholder, the
/// headers every request carries, and the request budget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlTemplateVectorTileSource {
    /// The `{z}/{x}/{y}` template the tileset publishes.
    pub url_template: String,
    /// Cache namespace, attribution, zoom span, wire encoding and style hints
    /// of the served tileset.
    pub source_descriptor: VectorTileSourceDescriptor,
    /// Terms the tileset's usage policy requires alongside attribution.
    /// Empty when the source declares none.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub terms_text: String,
    /// Values substituted for the template's optional `{s}` placeholder, in
    /// the source's preference order. Empty when the template has no `{s}`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subdomains: Vec<String>,
    /// Headers every tile request carries, in the order given. A credential
    /// a public template needs belongs here or in the template itself; the
    /// provider has no separate credential seam.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
    /// Request budget the source declares. `None` leaves requests unpaced;
    /// `Some` installs a token bucket on the shared client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pacing: Option<TileSourcePacing>,
}

/// Serves any web-mercator XYZ vector tileset that publishes a
/// `{z}/{x}/{y}` URL template.
///
/// Construct one with [`Self::new`] on the target's default transport, or
/// [`Self::with_transport`] to inject a transport. Both validate the source
/// descriptor and the template before any request is sent, so a malformed
/// source never reaches the network.
pub struct UrlTemplateVectorTileProvider {
    url_template: UrlTemplate,
    source: UrlTemplateVectorTileSource,
    client: Arc<UrlTemplateVectorClient>,
}

impl UrlTemplateVectorTileProvider {
    /// Builds a provider on the target's default transport.
    ///
    /// # Parameters
    /// - `source` — the tileset template, descriptor, subdomains, headers and
    ///   optional pacing.
    ///
    /// # Returns
    /// A provider ready to fetch tiles.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the descriptor names no
    /// source, declares an inverted zoom range or an unusable pacing, or when
    /// the template is malformed. See [`UrlTemplate::new`] for the template
    /// rules.
    pub fn new(source: UrlTemplateVectorTileSource) -> EveryMapResult<Self> {
        Self::build(source, None)
    }

    /// Builds a provider that executes requests through an explicit
    /// transport.
    ///
    /// # Parameters
    /// - `source` — the tileset template, descriptor, subdomains, headers and
    ///   optional pacing.
    /// - `transport` — the transport that sends each request. Hosts inject
    ///   their platform transport and tests inject a fake.
    ///
    /// # Returns
    /// A provider ready to fetch tiles through the supplied transport.
    ///
    /// # Errors
    /// Returns the same validation errors as [`Self::new`].
    pub fn with_transport(
        source: UrlTemplateVectorTileSource,
        transport: Arc<dyn HttpTransport>,
    ) -> EveryMapResult<Self> {
        Self::build(source, Some(transport))
    }

    /// The declaration this instance serves.
    pub fn source(&self) -> &UrlTemplateVectorTileSource {
        &self.source
    }

    /// Validates the declaration, builds the template and installs pacing.
    fn build(
        source: UrlTemplateVectorTileSource,
        transport: Option<Arc<dyn HttpTransport>>,
    ) -> EveryMapResult<Self> {
        let descriptor = &source.source_descriptor;
        validate_tile_source_declaration(
            &descriptor.source_name,
            descriptor.minimum_zoom_level,
            descriptor.maximum_zoom_level,
            source.pacing,
        )?;
        let url_template = UrlTemplate::new(source.url_template.clone(), &source.subdomains)?;
        let mut client = match transport {
            Some(transport) => UrlTemplateVectorClient::with_transport(transport),
            None => UrlTemplateVectorClient::keyless(),
        };
        if let Some(pacing) = source.pacing {
            let mut token_bucket =
                TokenBucket::new(pacing.capacity_tokens, pacing.refill_tokens_per_second);
            if let Some(maximum_wait_seconds) = pacing.maximum_wait_seconds {
                token_bucket =
                    token_bucket.with_maximum_wait(Duration::from_secs_f64(maximum_wait_seconds));
            }
            client.set_rate_limiter(Arc::new(token_bucket));
        }
        Ok(Self {
            url_template,
            source,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl VectorTileProvider for UrlTemplateVectorTileProvider {
    /// Describes the tileset this provider serves.
    fn source_descriptor(&self) -> VectorTileSourceDescriptor {
        self.source.source_descriptor.clone()
    }

    /// Fetches one web-mercator XYZ vector tile in its wire encoding.
    ///
    /// # Parameters
    /// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
    /// - `zoom_level` — the web-mercator zoom level of the tile.
    ///
    /// # Returns
    /// The tile bytes together with the encoding they arrived in. The shared
    /// [`VectorTileData::from_wire_bytes`] classifier identifies gzip-wrapped
    /// payloads from their magic bytes.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the tile is outside the
    /// served zoom span or index range, or when the payload is empty.
    /// Returns [`EveryMapError::HttpError`] for a non-success status,
    /// [`EveryMapError::RateLimited`] for HTTP 429, and
    /// [`EveryMapError::TransportError`] or [`EveryMapError::ClientError`]
    /// when no response arrives.
    async fn fetch_vector_tile(
        &self,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<VectorTileData> {
        let descriptor = &self.source.source_descriptor;
        validate_tile_coordinates(
            tile_x,
            tile_y,
            zoom_level,
            descriptor.minimum_zoom_level,
            descriptor.maximum_zoom_level,
            &descriptor.source_name,
        )?;
        let url = self.url_template.expand(tile_x, tile_y, zoom_level);
        let mut request = self.client.build_request(Method::Get, &url);
        for (name, value) in &self.source.headers {
            request = request.header(name.clone(), value.clone());
        }
        let response = self.client.request(request).await?;
        VectorTileData::from_wire_bytes(tile_x, tile_y, zoom_level, response.into_body())
    }
}

#[cfg(test)]
mod tests {
    use super::{UrlTemplateVectorTileProvider, UrlTemplateVectorTileSource};
    use crate::pacing::TileSourcePacing;
    use async_trait::async_trait;
    use everymap_core::domains::tiling::{
        VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor,
    };
    use everymap_core::error::{EveryMapError, EveryMapResult};
    use everymap_core::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// Smallest plausible MVT envelope (the repeated `layers` field key).
    const SAMPLE_TILE_BYTES: [u8; 2] = [0x1a, 0x00];

    /// One request seen by the fake transport, with the instant it arrived.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedRequest {
        url: String,
        headers: Vec<(String, String)>,
        received_at: Instant,
    }

    /// Transport that answers every request with a fixed body and records the
    /// request, so a test can prove URL, header and pacing behaviour without a
    /// server.
    struct RecordingTransport {
        requests: Mutex<Vec<RecordedRequest>>,
    }

    impl RecordingTransport {
        fn new() -> Self {
            Self {
                requests: Mutex::new(Vec::new()),
            }
        }

        fn requests(&self) -> Vec<RecordedRequest> {
            self.requests
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        }
    }

    #[async_trait]
    impl HttpTransport for RecordingTransport {
        async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
            self.requests
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(RecordedRequest {
                    url: request.url_with_query(),
                    headers: request.headers().to_vec(),
                    received_at: Instant::now(),
                });
            Ok(HttpResponse::new(
                200,
                HttpHeaders::default(),
                SAMPLE_TILE_BYTES.to_vec(),
            ))
        }
    }

    fn source_descriptor() -> VectorTileSourceDescriptor {
        VectorTileSourceDescriptor {
            source_name: "test/url-template".to_string(),
            attribution_text: "Test tiles".to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 15,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["openmaptiles".to_string()],
        }
    }

    fn source() -> UrlTemplateVectorTileSource {
        UrlTemplateVectorTileSource {
            url_template: "https://tiles.example.test/{z}/{x}/{y}.mvt".to_string(),
            source_descriptor: source_descriptor(),
            terms_text: "Test terms".to_string(),
            subdomains: Vec::new(),
            headers: vec![("x-api-version".to_string(), "2".to_string())],
            pacing: None,
        }
    }

    fn provider_with_transport(
        transport: Arc<RecordingTransport>,
    ) -> UrlTemplateVectorTileProvider {
        UrlTemplateVectorTileProvider::with_transport(source(), transport)
            .expect("the sample source is valid")
    }

    #[test]
    fn the_declaration_is_returned_verbatim() {
        let provider = provider_with_transport(Arc::new(RecordingTransport::new()));
        assert_eq!(provider.source(), &source());
        assert_eq!(provider.source_descriptor(), source_descriptor());
    }

    #[tokio::test]
    async fn every_fetch_carries_the_declared_headers() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        provider
            .fetch_vector_tile(2, 1, 3)
            .await
            .expect("the fake transport answers a tile");
        assert_eq!(
            transport.requests()[0].headers,
            vec![("x-api-version".to_string(), "2".to_string())]
        );
        assert_eq!(
            transport.requests()[0].url,
            "https://tiles.example.test/3/2/1.mvt"
        );
    }

    #[tokio::test]
    async fn a_declared_pacing_bucket_delays_the_second_fetch() {
        let transport = Arc::new(RecordingTransport::new());
        let injected_transport: Arc<dyn HttpTransport> = transport.clone();
        let provider = UrlTemplateVectorTileProvider::with_transport(
            UrlTemplateVectorTileSource {
                pacing: Some(TileSourcePacing::new(1, 200.0)),
                ..source()
            },
            injected_transport,
        )
        .expect("a strict but usable declaration builds");

        provider
            .fetch_vector_tile(2, 1, 3)
            .await
            .expect("the full bucket admits the first fetch");
        provider
            .fetch_vector_tile(2, 1, 3)
            .await
            .expect("the refilled bucket admits the second fetch");

        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        let gap = requests[1]
            .received_at
            .duration_since(requests[0].received_at);
        assert!(
            gap >= Duration::from_millis(4),
            "a one-token bucket refilling at 200/s must pace the second fetch; gap was {gap:?}"
        );
    }

    #[tokio::test]
    async fn gzip_wrapped_wire_bytes_report_the_gzip_encoding() {
        const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b];
        // The body is fixed per transport, so build one that returns gzip.
        struct GzipTransport;
        #[async_trait]
        impl HttpTransport for GzipTransport {
            async fn send(&self, _request: HttpRequest) -> EveryMapResult<HttpResponse> {
                Ok(HttpResponse::new(
                    200,
                    HttpHeaders::default(),
                    GZIP_MAGIC_BYTES.to_vec(),
                ))
            }
        }
        let provider =
            UrlTemplateVectorTileProvider::with_transport(source(), Arc::new(GzipTransport))
                .unwrap();
        let tile = provider.fetch_vector_tile(2, 1, 3).await.unwrap();
        assert_eq!(
            tile.tile_encoding,
            VectorTileEncoding::GzippedMapboxVectorTile
        );
    }

    #[tokio::test]
    async fn an_empty_payload_is_a_validation_error() {
        struct EmptyTransport;
        #[async_trait]
        impl HttpTransport for EmptyTransport {
            async fn send(&self, _request: HttpRequest) -> EveryMapResult<HttpResponse> {
                Ok(HttpResponse::new(200, HttpHeaders::default(), Vec::new()))
            }
        }
        let provider =
            UrlTemplateVectorTileProvider::with_transport(source(), Arc::new(EmptyTransport))
                .unwrap();
        assert!(matches!(
            provider.fetch_vector_tile(2, 1, 3).await,
            Err(EveryMapError::ValidationError(_))
        ));
    }

    #[tokio::test]
    async fn zoom_outside_the_declared_bounds_is_rejected_without_a_request() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        assert!(matches!(
            provider.fetch_vector_tile(0, 0, 16).await,
            Err(EveryMapError::ValidationError(_))
        ));
        assert_eq!(transport.requests().len(), 0);
    }

    #[tokio::test]
    async fn tile_indexes_outside_the_zoom_range_are_rejected_without_a_request() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        // At zoom 3 only 0..=7 indexes exist; 8 is out of range.
        assert!(matches!(
            provider.fetch_vector_tile(8, 1, 3).await,
            Err(EveryMapError::ValidationError(_))
        ));
        assert!(matches!(
            provider.fetch_vector_tile(1, 8, 3).await,
            Err(EveryMapError::ValidationError(_))
        ));
        assert_eq!(transport.requests().len(), 0);
    }

    #[test]
    fn a_blank_source_name_is_refused_at_construction() {
        let error = UrlTemplateVectorTileProvider::with_transport(
            UrlTemplateVectorTileSource {
                source_descriptor: VectorTileSourceDescriptor {
                    source_name: "  ".to_string(),
                    ..source_descriptor()
                },
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("a blank source name must be refused");
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn an_inverted_zoom_range_is_refused_at_construction() {
        let error = UrlTemplateVectorTileProvider::with_transport(
            UrlTemplateVectorTileSource {
                source_descriptor: VectorTileSourceDescriptor {
                    minimum_zoom_level: 14,
                    maximum_zoom_level: 6,
                    ..source_descriptor()
                },
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("an inverted zoom range must be refused");
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("14..=6"), "{detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn an_unusable_declared_pacing_is_refused_at_construction() {
        let error = UrlTemplateVectorTileProvider::with_transport(
            UrlTemplateVectorTileSource {
                pacing: Some(TileSourcePacing::new(0, 1.0)),
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("a zero capacity must be refused");
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn an_unknown_placeholder_is_refused_at_construction() {
        let error = UrlTemplateVectorTileProvider::with_transport(
            UrlTemplateVectorTileSource {
                url_template: "https://tiles.example.test/{z}/{x}/{y}.mvt?q={q}".to_string(),
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("an unknown placeholder must be refused");
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }
}
