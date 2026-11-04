//! Named public vector-tile sources.
//!
//! These constructors carry the templates, attribution and terms the Rustic
//! Maps SDK registry publishes for the keyless vector base maps, so a caller
//! names a source instead of repeating a template. Each returns a ready
//! [`UrlTemplateVectorTileSource`] for [`crate::UrlTemplateVectorTileProvider`].

use crate::provider::UrlTemplateVectorTileSource;
use everymap_core::domains::tiling::{VectorTileEncoding, VectorTileSourceDescriptor};

/// OpenStreetMap US publishes the Sourdough tileset, a plain MVT pyramid
/// capped at zoom 15.
pub fn openstreetmap_us_vector_source() -> UrlTemplateVectorTileSource {
    UrlTemplateVectorTileSource {
        url_template: "https://tiles.openstreetmap.us/vector/sourdough/{z}/{x}/{y}.mvt"
            .to_string(),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "openstreetmap-us/sourdough".to_string(),
            attribution_text:
                "Map data © OpenStreetMap contributors (ODbL) — vector tiles © OpenStreetMap US"
                    .to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 15,
            tile_encoding: VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["carto".to_string()],
        },
        terms_text:
            "OpenStreetMap US tileservice terms apply: free for low-volume, non-commercial use with attribution."
                .to_string(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    }
}

/// OpenFreeMap publishes the OpenMapTiles planet tileset, a gzip-wrapped MVT
/// pyramid capped at zoom 14.
pub fn openfreemap_vector_source() -> UrlTemplateVectorTileSource {
    UrlTemplateVectorTileSource {
        url_template: "https://tiles.openfreemap.org/planet/latest/{z}/{x}/{y}.pbf"
            .to_string(),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "openfreemap/planet".to_string(),
            attribution_text:
                "Map data © OpenStreetMap contributors (ODbL) — vector tiles © OpenFreeMap and OpenMapTiles"
                    .to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 14,
            tile_encoding: VectorTileEncoding::GzippedMapboxVectorTile,
            style_hints: vec![
                "openmaptiles".to_string(),
                "bright".to_string(),
                "fiord".to_string(),
                "positron".to_string(),
            ],
        },
        terms_text:
            "OpenFreeMap terms apply: free public use with attribution; bulk downloads are limited to 3,000 tiles or 50 MB per download."
                .to_string(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_named_sources_carry_their_templates_and_terms() {
        let openstreetmap_us = openstreetmap_us_vector_source();
        assert!(openstreetmap_us.url_template.contains("{z}/{x}/{y}"));
        assert_eq!(openstreetmap_us.source_descriptor.maximum_zoom_level, 15);
        assert!(!openstreetmap_us
            .source_descriptor
            .attribution_text
            .is_empty());
        let openfreemap = openfreemap_vector_source();
        assert!(openfreemap.url_template.contains("{z}/{x}/{y}"));
        assert_eq!(openfreemap.source_descriptor.maximum_zoom_level, 14);
        assert!(!openfreemap.terms_text.is_empty());
    }
}
