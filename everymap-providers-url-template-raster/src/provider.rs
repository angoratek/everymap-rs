//! The provider that serves one URL-template raster tileset.
//!
//! One instance serves one raster tileset: the validated template turns tile
//! coordinates into request URLs, the source declaration names the tileset and
//! records the image format and resolution it publishes, and every fetch is
//! paced by the source's declared token bucket and retried in bounds through
//! the shared client.

use crate::client::UrlTemplateRasterClient;
use async_trait::async_trait;
use everymap_core::client::TokenBucket;
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileData, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
use everymap_core::error::{EveryMapError, EveryMapResult};
use everymap_core::http::{HttpTransport, Method};
use everymap_providers_url_template_vector::url_template::UrlTemplate;
use everymap_providers_url_template_vector::TileSourcePacing;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Highest zoom level whose full XYZ index range still fits a `u32`
/// (`2^zoom_level - 1 <= u32::MAX`); at deeper zooms every `u32` tile index
/// the contract carries is valid, so the index bound check does not apply.
const MAXIMUM_ZOOM_LEVEL_WITH_BOUNDED_TILE_INDICES: u8 = 32;

/// What one URL-template raster source serves.
///
/// The declaration names the tileset, records its attribution and terms, and
/// describes the imagery it publishes. It also controls how requests are
/// built: the template, the values substituted for its optional `{s}`
/// placeholder, the headers every request carries, and the request budget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlTemplateRasterTileSource {
    /// The `{z}/{x}/{y}` template the tileset publishes.
    pub url_template: String,
    /// The tileset's name, used verbatim in every diagnostic it raises.
    pub source_name: String,
    /// Human-readable attribution the tileset's usage terms require wherever
    /// the imagery is shown.
    pub attribution_text: String,
    /// Terms the tileset's usage policy requires alongside attribution.
    /// Empty when the source declares none.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub terms_text: String,
    /// Lowest zoom level the tileset serves.
    pub minimum_zoom_level: u8,
    /// Highest zoom level the tileset serves.
    pub maximum_zoom_level: u8,
    /// Image encoding the tileset publishes.
    pub image_format: RasterImageFormat,
    /// Pixel resolution the tileset publishes.
    pub tile_resolution: RasterTileResolution,
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

/// Serves any web-mercator XYZ raster tileset that publishes a
/// `{z}/{x}/{y}` URL template.
///
/// Construct one with [`Self::new`] on the target's default transport, or
/// [`Self::with_transport`] to inject a transport. Both validate the source
/// declaration and the template before any request is sent, so a malformed
/// source never reaches the network.
pub struct UrlTemplateRasterTileProvider {
    url_template: UrlTemplate,
    source: UrlTemplateRasterTileSource,
    client: Arc<UrlTemplateRasterClient>,
}

impl UrlTemplateRasterTileProvider {
    /// Builds a provider on the target's default transport.
    ///
    /// # Parameters
    /// - `source` — the tileset template, declaration, subdomains, headers and
    ///   optional pacing.
    ///
    /// # Returns
    /// A provider ready to fetch tiles.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the declaration names
    /// no source, declares an inverted zoom range or an unusable pacing, or
    /// when the template is malformed. See [`UrlTemplate::new`] for the
    /// template rules.
    pub fn new(source: UrlTemplateRasterTileSource) -> EveryMapResult<Self> {
        Self::build(source, None)
    }

    /// Builds a provider that executes requests through an explicit
    /// transport.
    ///
    /// # Parameters
    /// - `source` — the tileset template, declaration, subdomains, headers and
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
        source: UrlTemplateRasterTileSource,
        transport: Arc<dyn HttpTransport>,
    ) -> EveryMapResult<Self> {
        Self::build(source, Some(transport))
    }

    /// The declaration this instance serves.
    pub fn source(&self) -> &UrlTemplateRasterTileSource {
        &self.source
    }

    /// Validates the declaration, builds the template and installs pacing.
    fn build(
        source: UrlTemplateRasterTileSource,
        transport: Option<Arc<dyn HttpTransport>>,
    ) -> EveryMapResult<Self> {
        validate_source(&source)?;
        let url_template = UrlTemplate::new(source.url_template.clone(), &source.subdomains)?;
        let mut client = match transport {
            Some(transport) => UrlTemplateRasterClient::with_transport(transport),
            None => UrlTemplateRasterClient::keyless(),
        };
        if let Some(pacing) = source.pacing {
            client.set_rate_limiter(Arc::new(TokenBucket::new(
                pacing.capacity_tokens,
                pacing.refill_tokens_per_second,
            )));
        }
        Ok(Self {
            url_template,
            source,
            client: Arc::new(client),
        })
    }

    /// Rejects tile coordinates the declared tileset cannot serve, before the
    /// rate limiter or the network is touched.
    fn validate_tile_coordinates(
        &self,
        tile_x: u32,
        tile_y: u32,
        zoom_level: u8,
    ) -> EveryMapResult<()> {
        if zoom_level < self.source.minimum_zoom_level
            || zoom_level > self.source.maximum_zoom_level
        {
            return Err(EveryMapError::ValidationError(format!(
                "zoom level {zoom_level} is outside the {}..={} range the {} raster tile source \
                 declares",
                self.source.minimum_zoom_level,
                self.source.maximum_zoom_level,
                self.source.source_name
            )));
        }
        if zoom_level < MAXIMUM_ZOOM_LEVEL_WITH_BOUNDED_TILE_INDICES {
            let maximum_tile_index = (1u32 << zoom_level) - 1;
            if tile_x > maximum_tile_index || tile_y > maximum_tile_index {
                return Err(EveryMapError::ValidationError(format!(
                    "tile x={tile_x} y={tile_y} exceeds the {maximum_tile_index} maximum at zoom \
                     {zoom_level}"
                )));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl RasterTileProvider for UrlTemplateRasterTileProvider {
    /// Fetches one web-mercator XYZ raster tile.
    ///
    /// # Parameters
    /// - `request` — tile coordinates, image format and resolution.
    ///
    /// # Returns
    /// The encoded tile bytes together with the format and resolution the
    /// caller requested, so a cache key or renderer never re-derives them.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the tile is outside the
    /// served zoom span or index range. Returns
    /// [`EveryMapError::HttpError`] for a non-success status,
    /// [`EveryMapError::RateLimited`] for HTTP 429, and
    /// [`EveryMapError::TransportError`] or [`EveryMapError::ClientError`]
    /// when no response arrives.
    async fn fetch_raster_tile(
        &self,
        request: &RasterTileRequest,
    ) -> EveryMapResult<RasterTileData> {
        self.validate_tile_coordinates(request.tile_x, request.tile_y, request.zoom_level)?;
        let url = self
            .url_template
            .expand(request.tile_x, request.tile_y, request.zoom_level);
        let mut http_request = self.client.build_request(Method::Get, &url);
        for (name, value) in &self.source.headers {
            http_request = http_request.header(name.clone(), value.clone());
        }
        let response = self.client.request(http_request).await?;
        Ok(RasterTileData::for_request(request, response.into_body()))
    }
}

/// Validates the parts of the declaration the template validator cannot see.
fn validate_source(source: &UrlTemplateRasterTileSource) -> EveryMapResult<()> {
    if source.source_name.trim().is_empty() {
        return Err(EveryMapError::ValidationError(
            "the raster tile source name must be non-empty".to_string(),
        ));
    }
    if source.minimum_zoom_level > source.maximum_zoom_level {
        return Err(EveryMapError::ValidationError(format!(
            "the {} raster tile source declares the inverted zoom range {}..={}",
            source.source_name, source.minimum_zoom_level, source.maximum_zoom_level
        )));
    }
    if let Some(pacing) = source.pacing {
        pacing.validate(&source.source_name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{UrlTemplateRasterTileProvider, UrlTemplateRasterTileSource};
    use async_trait::async_trait;
    use everymap_core::domains::tiling::{
        RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
    };
    use everymap_core::error::{EveryMapError, EveryMapResult};
    use everymap_core::http::{HttpHeaders, HttpRequest, HttpResponse, HttpTransport};
    use everymap_providers_url_template_vector::TileSourcePacing;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// Smallest plausible PNG envelope (the 8-byte signature).
    const SAMPLE_TILE_BYTES: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

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

    fn source() -> UrlTemplateRasterTileSource {
        UrlTemplateRasterTileSource {
            url_template: "https://tiles.example.test/{z}/{x}/{y}.png".to_string(),
            source_name: "test/url-template".to_string(),
            attribution_text: "Test imagery".to_string(),
            terms_text: "Test terms".to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 15,
            image_format: RasterImageFormat::Png,
            tile_resolution: RasterTileResolution::Standard,
            subdomains: Vec::new(),
            headers: vec![("x-api-version".to_string(), "2".to_string())],
            pacing: None,
        }
    }

    fn request() -> RasterTileRequest {
        RasterTileRequest::new(
            2,
            1,
            3,
            RasterImageFormat::Jpeg,
            RasterTileResolution::Retina,
        )
    }

    fn provider_with_transport(
        transport: Arc<RecordingTransport>,
    ) -> UrlTemplateRasterTileProvider {
        UrlTemplateRasterTileProvider::with_transport(source(), transport)
            .expect("the sample source is valid")
    }

    #[test]
    fn the_declaration_is_returned_verbatim() {
        let provider = provider_with_transport(Arc::new(RecordingTransport::new()));
        assert_eq!(provider.source(), &source());
    }

    #[tokio::test]
    async fn a_fetch_echoes_the_requested_format_and_resolution() {
        let provider = provider_with_transport(Arc::new(RecordingTransport::new()));
        let tile = provider
            .fetch_raster_tile(&request())
            .await
            .expect("the fake transport answers a tile");
        assert_eq!(tile.tile_x, 2);
        assert_eq!(tile.tile_y, 1);
        assert_eq!(tile.zoom_level, 3);
        assert_eq!(tile.image_format, RasterImageFormat::Jpeg);
        assert_eq!(tile.tile_resolution, RasterTileResolution::Retina);
        assert_eq!(tile.image_bytes, SAMPLE_TILE_BYTES.to_vec());
    }

    #[tokio::test]
    async fn every_fetch_carries_the_declared_headers() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        provider
            .fetch_raster_tile(&request())
            .await
            .expect("the fake transport answers a tile");
        assert_eq!(
            transport.requests()[0].headers,
            vec![("x-api-version".to_string(), "2".to_string())]
        );
        assert_eq!(
            transport.requests()[0].url,
            "https://tiles.example.test/3/2/1.png"
        );
    }

    #[tokio::test]
    async fn a_declared_pacing_bucket_delays_the_second_fetch() {
        let transport = Arc::new(RecordingTransport::new());
        let injected_transport: Arc<dyn HttpTransport> = transport.clone();
        let provider = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
                pacing: Some(TileSourcePacing::new(1, 200.0)),
                ..source()
            },
            injected_transport,
        )
        .expect("a strict but usable declaration builds");

        provider
            .fetch_raster_tile(&request())
            .await
            .expect("the full bucket admits the first fetch");
        provider
            .fetch_raster_tile(&request())
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
    async fn zoom_outside_the_declared_bounds_is_rejected_without_a_request() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        let outside = RasterTileRequest::new(
            0,
            0,
            16,
            RasterImageFormat::Png,
            RasterTileResolution::Standard,
        );
        assert!(matches!(
            provider.fetch_raster_tile(&outside).await,
            Err(EveryMapError::ValidationError(_))
        ));
        assert_eq!(transport.requests().len(), 0);
    }

    #[tokio::test]
    async fn tile_indexes_outside_the_zoom_range_are_rejected_without_a_request() {
        let transport = Arc::new(RecordingTransport::new());
        let provider = provider_with_transport(Arc::clone(&transport));
        // At zoom 3 only 0..=7 indexes exist; 8 is out of range.
        for (tile_x, tile_y) in [(8, 1), (1, 8)] {
            let outside = RasterTileRequest::new(
                tile_x,
                tile_y,
                3,
                RasterImageFormat::Png,
                RasterTileResolution::Standard,
            );
            assert!(matches!(
                provider.fetch_raster_tile(&outside).await,
                Err(EveryMapError::ValidationError(_))
            ));
        }
        assert_eq!(transport.requests().len(), 0);
    }

    #[test]
    fn a_blank_source_name_is_refused_at_construction() {
        let error = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
                source_name: "  ".to_string(),
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
        let error = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
                minimum_zoom_level: 12,
                maximum_zoom_level: 8,
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("an inverted zoom range must be refused");
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("12..=8"), "{detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn an_unusable_declared_pacing_is_refused_at_construction() {
        let error = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
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
        let error = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
                url_template: "https://tiles.example.test/{z}/{x}/{y}.png?q={q}".to_string(),
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("an unknown placeholder must be refused");
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn a_subdomain_placeholder_without_values_is_refused_at_construction() {
        let error = UrlTemplateRasterTileProvider::with_transport(
            UrlTemplateRasterTileSource {
                url_template: "https://{s}.tiles.example.test/{z}/{x}/{y}.png".to_string(),
                ..source()
            },
            Arc::new(RecordingTransport::new()),
        )
        .err()
        .expect("a subdomain placeholder needs subdomains");
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }
}
