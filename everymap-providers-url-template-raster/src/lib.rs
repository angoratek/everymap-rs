//! A provider-neutral raster-tile source driven by a `{z}/{x}/{y}` URL
//! template.
//!
//! [`UrlTemplateRasterTileProvider`] implements
//! [`everymap_core::domains::tiling::RasterTileProvider`] for any web-mercator
//! XYZ raster tileset that publishes a URL template. The template's authority,
//! path layout and file extension all come from the caller, so a base map is a
//! template plus a [`UrlTemplateRasterTileSource`], never a new provider crate.
//!
//! # Template rules
//!
//! A template must carry the `{z}`, `{x}` and `{y}` placeholders and use the
//! `http` or `https` scheme. It may also carry `{s}`, which is substituted
//! with one of the source's declared subdomains or schemes; a tile always maps
//! to the same value. Any other placeholder is refused at construction with
//! [`everymap_core::error::EveryMapError::ValidationError`]. The template
//! validator and the pacing vocabulary are shared with the vector source, so
//! both tile contracts agree on what a valid template is.
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
//! Each [`everymap_core::domains::tiling::RasterTileData`] echoes the format
//! and resolution of the request that produced it, so a cache key or renderer
//! never has to re-derive what the bytes are.
//!
//! ```no_run
//! use everymap_core::domains::tiling::{
//!     RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
//! };
//! use everymap_providers_url_template_raster::{
//!     UrlTemplateRasterTileProvider, UrlTemplateRasterTileSource,
//! };
//!
//! # async fn example() -> Result<(), everymap_core::error::EveryMapError> {
//! let provider = UrlTemplateRasterTileProvider::new(UrlTemplateRasterTileSource {
//!     url_template: "https://{s}.tiles.example.com/{z}/{x}/{y}.png".to_string(),
//!     source_name: "example/imagery".to_string(),
//!     attribution_text: "Imagery © example".to_string(),
//!     terms_text: String::new(),
//!     minimum_zoom_level: 0,
//!     maximum_zoom_level: 18,
//!     image_format: RasterImageFormat::Png,
//!     tile_resolution: RasterTileResolution::Standard,
//!     subdomains: vec!["a".to_string(), "b".to_string()],
//!     headers: Vec::new(),
//!     pacing: None,
//! })?;
//! let tile = provider
//!     .fetch_raster_tile(&RasterTileRequest::new(
//!         1,
//!         2,
//!         3,
//!         RasterImageFormat::Png,
//!         RasterTileResolution::Standard,
//!     ))
//!     .await?;
//! # let _ = tile;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod provider;

pub use client::URL_TEMPLATE_RASTER_PROVIDER_NAME;
pub use everymap_providers_url_template_vector::TileSourcePacing;
pub use provider::{UrlTemplateRasterTileProvider, UrlTemplateRasterTileSource};
