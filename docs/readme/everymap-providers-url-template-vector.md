# everymap-providers-url-template-vector

A provider-neutral vector-tile provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs), serving any web-mercator XYZ tileset that publishes a `{z}/{x}/{y}` URL template.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Vector tiles | `VectorTileProvider` | `UrlTemplateVectorTileProvider` |

Not served: geocoding, place search, type-ahead, routing, distance matrix, traffic, raster tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

## Installation

```bash
cargo add everymap-providers-url-template-vector everymap-core
```

## Quick start

```rust
use everymap_core::domains::tiling::{VectorTileProvider, VectorTileSourceDescriptor};
use everymap_providers_url_template_vector::{
    UrlTemplateVectorTileProvider, UrlTemplateVectorTileSource,
};

#[tokio::main]
async fn main() {
    let provider = UrlTemplateVectorTileProvider::new(UrlTemplateVectorTileSource {
        url_template: "https://tiles.example.com/{z}/{x}/{y}.mvt".to_string(),
        source_descriptor: VectorTileSourceDescriptor {
            source_name: "example/streets".to_string(),
            attribution_text: "Map data © example".to_string(),
            minimum_zoom_level: 0,
            maximum_zoom_level: 14,
            tile_encoding:
                everymap_core::domains::tiling::VectorTileEncoding::MapboxVectorTile,
            style_hints: vec!["bright".to_string()],
        },
        terms_text: String::new(),
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    })
    .expect("the source declaration is valid");
    let tile = provider.fetch_vector_tile(1, 2, 3).await.expect("the tile is served");
    println!("{:?} with {} bytes", tile.tile_encoding, tile.tile_bytes.len());
}
```

## Notes

A tile keeps its wire encoding; the response sniffs the gzip magic bytes and refuses an empty body, so a caller never decodes under the wrong assumption. The crate also ships `openstreetmap_us_vector_source()` and `openfreemap_vector_source()`, the named public tilesets with their templates, attribution, terms and encodings.
