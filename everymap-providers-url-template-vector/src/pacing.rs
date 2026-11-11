//! Token-bucket pacing a vector tile source declares for its own request
//! budget.
//!
//! The vocabulary and validation live in [`everymap_core::tile_source`],
//! shared with the URL-template raster provider, so both tile contracts read
//! a declared budget the same way. This module re-exports the type under its
//! original path.

pub use everymap_core::tile_source::{
    TileSourcePacing, DEFAULT_CAPACITY_TOKENS, DEFAULT_REFILL_TOKENS_PER_SECOND,
};
