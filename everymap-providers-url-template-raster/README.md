# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-url-template-raster

A provider-neutral raster-tile provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs), serving any web-mercator XYZ tileset that publishes a `{z}/{x}/{y}` URL template.

## Domain coverage

| Domain | Core trait | Supported |
|---|---|---|
| Raster tiles | `RasterTileProvider` | ✅ |

## Installation

```bash
cargo add everymap-providers-url-template-raster everymap-core
```

## Quick start

```rust
use everymap_core::domains::tiling::{RasterImageFormat, RasterTileProvider, RasterTileResolution};
use everymap_providers_url_template_raster::{
    UrlTemplateRasterTileProvider, UrlTemplateRasterTileSource,
};

#[tokio::main]
async fn main() {
    let provider = UrlTemplateRasterTileProvider::new(UrlTemplateRasterTileSource {
        url_template: "https://tile.openstreetmap.org/{z}/{x}/{y}.png".to_string(),
        source_name: "openstreetmap/standard".to_string(),
        attribution_text: "Map data © OpenStreetMap contributors (ODbL)".to_string(),
        terms_text: "OpenStreetMap tile usage policy applies: low-volume use with attribution."
            .to_string(),
        minimum_zoom_level: 0,
        maximum_zoom_level: 19,
        image_format: RasterImageFormat::Png,
        tile_resolution: RasterTileResolution::Standard,
        subdomains: Vec::new(),
        headers: Vec::new(),
        pacing: None,
    })
    .expect("the source declaration is valid");
    let tile = provider.fetch_raster_tile(1, 2, 3).await.expect("the tile is served");
    println!("{} bytes", tile.tile_bytes.len());
}
```

## Notes

The template may carry `{s}`, substituted from the source's declared subdomains or schemes; any other placeholder, a non-http scheme or a blank subdomain list is refused at construction. Response tiles echo the requested format and resolution. A declared pacing budget installs a token bucket on the shared client; an undeclared budget leaves requests unpaced.
