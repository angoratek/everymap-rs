//! Named public vector-tile sources.
//!
//! These constructors carry the templates, attribution and terms published by
//! the tileset operators, so a caller names a source instead of repeating a
//! template. Each returns a ready [`UrlTemplateVectorTileSource`] for
//! [`crate::UrlTemplateVectorTileProvider`].

use crate::provider::UrlTemplateVectorTileSource;
use everymap_core::domains::tiling::{VectorTileEncoding, VectorTileSourceDescriptor};

/// `User-Agent` the OpenStreetMap US tileservice asks clients to identify
/// themselves with.
///
/// The service's usage policy requires an accurate identifying agent on every
/// request and refuses clients that send none, so the named source carries it
/// by default; a host may override the header in its own declaration.
pub const OPENSTREETMAP_US_USER_AGENT: &str =
    concat!("everymap-osmus-provider/", env!("CARGO_PKG_VERSION"));

/// Header the identifying user agent travels in.
const USER_AGENT_HEADER_NAME: &str = "User-Agent";

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
            "OpenStreetMap US tileservice terms apply: free for low-volume, non-commercial use; commercial or higher-volume use needs an approved partnership; requests must carry an accurate identifying User-Agent, must honor Cache-Control and must back off on HTTP 429; attribution to OSM US, OpenStreetMap and OpenMapTiles is required."
                .to_string(),
        subdomains: Vec::new(),
        headers: vec![(
            USER_AGENT_HEADER_NAME.to_string(),
            OPENSTREETMAP_US_USER_AGENT.to_string(),
        )],
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
        assert!(
            openstreetmap_us
                .headers
                .iter()
                .any(|(header_name, header_value)| header_name == "User-Agent"
                    && !header_value.is_empty()),
            "the tileservice's usage policy requires an identifying user agent"
        );
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
