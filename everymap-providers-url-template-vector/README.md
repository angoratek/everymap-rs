# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

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
