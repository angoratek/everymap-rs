# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-url-template-raster

A provider-neutral raster-tile provider for [EveryMap-RS](https://github.com/angoratek/everymap-rs), serving any web-mercator XYZ tileset that publishes a `{z}/{x}/{y}` URL template.

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Raster tiles | `RasterTileProvider` | `UrlTemplateRasterTileProvider` |

Not served: geocoding, place search, type-ahead, routing, distance matrix, traffic, vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

## Installation

```bash
cargo add everymap-providers-url-template-raster everymap-core
```

## Quick start

```rust
use everymap_core::domains::tiling::{
    RasterImageFormat, RasterTileProvider, RasterTileRequest, RasterTileResolution,
};
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
    let tile = provider
        .fetch_raster_tile(&RasterTileRequest::new(
            1,
            2,
            3,
            RasterImageFormat::Png,
            RasterTileResolution::Standard,
        ))
        .await
        .expect("the tile is served");
    println!("{} bytes", tile.image_bytes.len());
}
```

## Notes

The template may carry `{s}`, substituted from the source's declared subdomains or schemes; any other placeholder, a non-http scheme or a blank subdomain list is refused at construction. Response tiles echo the requested format and resolution. A declared pacing budget installs a token bucket on the shared client; an undeclared budget leaves requests unpaced.
