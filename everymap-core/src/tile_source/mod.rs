//! Shared vocabulary for provider-neutral `{z}/{x}/{y}` tile sources.
//!
//! Every URL-template tile source — raster or vector — accepts the same
//! `{z}/{x}/{y}` template, declares the same zoom span and request budget, and
//! rejects the same out-of-range tile coordinates. This module holds those
//! rules once, so the raster and vector providers cannot drift apart:
//!
//! - [`UrlTemplate`] validates and expands the template.
//! - [`TileSourcePacing`] describes the source's request budget.
//! - [`validate_tile_source_declaration`] checks the name, zoom span and
//!   pacing at construction.
//! - [`validate_tile_coordinates`] rejects a fetch outside the served zoom
//!   span or XYZ index range before the network is touched.

mod pacing;
mod url_template;

pub use pacing::{TileSourcePacing, DEFAULT_CAPACITY_TOKENS, DEFAULT_REFILL_TOKENS_PER_SECOND};
pub use url_template::UrlTemplate;

use crate::error::{EveryMapError, EveryMapResult};

/// Highest zoom level whose full XYZ index range still fits in a `u32`
/// (`2^zoom_level - 1 <= u32::MAX`); at deeper zooms every `u32` tile index
/// the contracts carry is valid, so the index bound check does not apply.
pub const MAXIMUM_ZOOM_LEVEL_WITH_BOUNDED_TILE_INDICES: u8 = 32;

/// Validates the declaration every URL-template tile source shares.
///
/// # Parameters
/// - `source_name` — the tileset's name, used verbatim in diagnostics.
/// - `minimum_zoom_level`, `maximum_zoom_level` — the inclusive zoom span the
///   source publishes.
/// - `pacing` — the optional request budget the source declares.
///
/// # Returns
/// `Ok(())` when the declaration can serve requests.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] when the name is blank, the
/// zoom span is inverted, or the pacing cannot be enforced.
pub fn validate_tile_source_declaration(
    source_name: &str,
    minimum_zoom_level: u8,
    maximum_zoom_level: u8,
    pacing: Option<TileSourcePacing>,
) -> EveryMapResult<()> {
    if source_name.trim().is_empty() {
        return Err(EveryMapError::ValidationError(
            "the tile source name must be non-empty".to_string(),
        ));
    }
    if minimum_zoom_level > maximum_zoom_level {
        return Err(EveryMapError::ValidationError(format!(
            "the {source_name} tile source declares the inverted zoom range \
             {minimum_zoom_level}..={maximum_zoom_level}"
        )));
    }
    if let Some(pacing) = pacing {
        pacing.validate(source_name)?;
    }
    Ok(())
}

/// Rejects tile coordinates the declared source cannot serve.
///
/// The check runs before a request is built, so an out-of-range tile never
/// touches the rate limiter or the network.
///
/// # Parameters
/// - `tile_x`, `tile_y` — the tile's column and row at `zoom_level`.
/// - `zoom_level` — the web-mercator zoom level of the tile.
/// - `minimum_zoom_level`, `maximum_zoom_level` — the inclusive zoom span the
///   source publishes.
/// - `source_name` — the tileset's name, used verbatim in the diagnostic.
///
/// # Returns
/// `Ok(())` when the source serves the tile.
///
/// # Errors
/// Returns [`EveryMapError::ValidationError`] when the zoom level is outside
/// the declared span or the column or row exceeds the `2^zoom_level` grid.
pub fn validate_tile_coordinates(
    tile_x: u32,
    tile_y: u32,
    zoom_level: u8,
    minimum_zoom_level: u8,
    maximum_zoom_level: u8,
    source_name: &str,
) -> EveryMapResult<()> {
    if zoom_level < minimum_zoom_level || zoom_level > maximum_zoom_level {
        return Err(EveryMapError::ValidationError(format!(
            "zoom level {zoom_level} is outside the {minimum_zoom_level}..={maximum_zoom_level} \
             range the {source_name} tile source declares"
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

#[cfg(test)]
mod tests {
    use super::{
        validate_tile_coordinates, validate_tile_source_declaration, TileSourcePacing,
        MAXIMUM_ZOOM_LEVEL_WITH_BOUNDED_TILE_INDICES,
    };
    use crate::error::EveryMapError;

    #[test]
    fn a_valid_declaration_is_accepted() {
        assert!(validate_tile_source_declaration("test/source", 0, 15, None).is_ok());
        assert!(validate_tile_source_declaration(
            "test/source",
            0,
            15,
            Some(TileSourcePacing::default())
        )
        .is_ok());
    }

    #[test]
    fn a_blank_source_name_is_refused() {
        let error = validate_tile_source_declaration("   ", 0, 15, None).unwrap_err();
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn an_inverted_zoom_range_names_the_source_and_the_range() {
        let error = validate_tile_source_declaration("test/inverted", 12, 8, None).unwrap_err();
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("test/inverted"), "{detail}");
                assert!(detail.contains("12..=8"), "{detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn an_unusable_declared_pacing_is_refused() {
        let error = validate_tile_source_declaration(
            "test/source",
            0,
            15,
            Some(TileSourcePacing::new(0, 1.0)),
        )
        .unwrap_err();
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn a_tile_inside_the_declared_span_is_accepted() {
        assert!(validate_tile_coordinates(7, 7, 3, 0, 15, "test/source").is_ok());
    }

    #[test]
    fn a_zoom_outside_the_declared_span_is_refused() {
        for zoom_level in [16, 20] {
            let error =
                validate_tile_coordinates(0, 0, zoom_level, 0, 15, "test/source").unwrap_err();
            match error {
                EveryMapError::ValidationError(detail) => {
                    assert!(detail.contains("0..=15"), "{detail}");
                }
                other => panic!("expected a validation error, got {other:?}"),
            }
        }
        let error = validate_tile_coordinates(0, 0, 1, 2, 15, "test/source").unwrap_err();
        assert!(matches!(error, EveryMapError::ValidationError(_)));
    }

    #[test]
    fn a_tile_index_outside_the_zoom_grid_is_refused() {
        // At zoom 3 only 0..=7 indexes exist; 8 is out of range.
        for (tile_x, tile_y) in [(8, 1), (1, 8)] {
            let error =
                validate_tile_coordinates(tile_x, tile_y, 3, 0, 15, "test/source").unwrap_err();
            match error {
                EveryMapError::ValidationError(detail) => {
                    assert!(detail.contains("7 maximum"), "{detail}");
                }
                other => panic!("expected a validation error, got {other:?}"),
            }
        }
    }

    #[test]
    fn every_tile_index_is_valid_at_the_deepest_boundable_zoom() {
        // At zoom 32 the XYZ range would need a 33-bit index, so the guard
        // must stop the bound check rather than shift past `u32::MAX`.
        let deepest = MAXIMUM_ZOOM_LEVEL_WITH_BOUNDED_TILE_INDICES;
        assert!(
            validate_tile_coordinates(u32::MAX, u32::MAX, deepest, 0, deepest, "test/source")
                .is_ok()
        );
    }
}
