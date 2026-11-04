# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-url-template-vector

A provider-neutral vector-tile provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs), serving any web-mercator XYZ tileset that publishes a `{z}/{x}/{y}` URL template.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Vector tiles | `VectorTileProvider` | ✅ |

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
