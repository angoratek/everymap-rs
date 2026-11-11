//! Validation and expansion of the `{z}/{x}/{y}` URL template a vector tileset
//! publishes.
//!
//! The rules and implementation live in [`everymap_core::tile_source`], shared
//! with the URL-template raster provider, so both tile contracts accept the
//! same templates. This module re-exports the type under its original path.

pub use everymap_core::tile_source::UrlTemplate;
