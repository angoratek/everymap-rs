//! A provider-neutral vector-tile source driven by a `{z}/{x}/{y}` URL
//! template.
//!
//! [`UrlTemplateVectorTileProvider`] implements
//! [`everymap_core::domains::tiling::VectorTileProvider`] for any web-mercator
//! XYZ tileset that publishes a URL template, the shape a TileJSON `tiles`
//! array names. The template's authority, path layout, file extension and
//! query string all come from the caller, so a new tileset is a template plus
//! a [`UrlTemplateVectorTileSource`], never a new provider crate.
//!
//! # Template rules
//!
//! A template must carry the `{z}`, `{x}` and `{y}` placeholders and use the
//! `http` or `https` scheme. It may also carry `{s}`, which is substituted
//! with one of the source's declared subdomains or schemes; a tile always maps
//! to the same value. Any other placeholder is refused at construction with
//! [`everymap_core::error::EveryMapError::ValidationError`].
//!
//! # Requests
//!
//! Each fetch is a `GET` through the shared client with the configured
//! headers applied. The source's optional [`TileSourcePacing`] installs a
//! token bucket on that client, so declared budgets throttle while an
//! undeclared budget leaves requests unpaced. Non-success statuses surface as
//! [`everymap_core::error::EveryMapError::HttpError`] and HTTP 429 as
//! [`everymap_core::error::EveryMapError::RateLimited`]; transient failures
//! are retried in bounds by the shared client.
//!
//! # Tiles
//!
//! Tiles are returned in their wire encoding. [`VectorTileData::from_wire_bytes`]
//! classifies the payload from its gzip magic bytes and refuses an empty body,
//! so callers never decode a tile under the wrong encoding assumption.
//!
//! ```no_run
//! use everymap_core::domains::tiling::{VectorTileEncoding, VectorTileProvider, VectorTileSourceDescriptor};
//! use everymap_providers_url_template_vector::{
//!     UrlTemplateVectorTileProvider, UrlTemplateVectorTileSource,
//! };
//!
//! # async fn example() -> Result<(), everymap_core::error::EveryMapError> {
//! let provider = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
//!     url_template: "https://{s}.tiles.example.com/{z}/{x}/{y}.mvt".to_string(),
//!     source_descriptor: VectorTileSourceDescriptor {
//!         source_name: "example/streets".to_string(),
//!         attribution_text: "Map data © example".to_string(),
//!         minimum_zoom_level: 0,
//!         maximum_zoom_level: 14,
//!         tile_encoding: VectorTileEncoding::GzippedMapboxVectorTile,
//!         style_hints: vec!["bright".to_string()],
//!     },
//!     terms_text: String::new(),
//!     subdomains: vec!["a".to_string(), "b".to_string()],
//!     headers: Vec::new(),
//!     pacing: None,
//! })?;
//! let tile = provider.fetch_vector_tile(1, 2, 3).await?;
//! # let _ = tile;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod pacing;
pub mod provider;
pub mod url_template;

pub use client::URL_TEMPLATE_VECTOR_PROVIDER_NAME;
pub use pacing::{TileSourcePacing, DEFAULT_CAPACITY_TOKENS, DEFAULT_REFILL_TOKENS_PER_SECOND};
pub use provider::{UrlTemplateVectorTileProvider, UrlTemplateVectorTileSource};
pub use url_template::UrlTemplate;
