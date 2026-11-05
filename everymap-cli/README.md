# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-cli

Unified command-line interface for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — one binary and 11 geospatial commands. It selects **HERE**, **Google**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **OSRM** and **Valhalla**; the URL-template raster and vector crates are available through the library. A provider answers a command it cannot serve with the documented unsupported-domain error.

## Installation

```bash
cargo install everymap-cli
```

Prebuilt binaries for 5 targets (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows) are attached to each [GitHub Release](https://github.com/angoratek/everymap-rs/releases).

## Commands

| Command | Purpose |
|---|---|
| `geocode` | Forward geocoding |
| `reverse-geocode` | Coordinates to address |
| `route` | Directions between two points |
| `traffic` | Traffic flow and incidents |
| `position` | Network positioning |
| `isoline` | Reachable range / isochrones |
| `match-route` | Map matching (snap to road) |
| `tour` | Waypoint optimization (TSP) |
| `tile` | Download a raster or vector map tile to a file (`--variant raster\|vector`) |
| `attributes` | Geodata attributes (e.g. HERE roads, buildings, landmarks) |
| `map-image` | Save a static map image to a file |

## Usage

Global flags (`--provider`, `--api-key`, `--output`, `--verbose`) must come **before** the subcommand:

```bash
# HERE provider (default)
everymap --api-key $HERE_KEY geocode "Brandenburg Gate, Berlin"
everymap --api-key $HERE_KEY route --origin "52.52,13.405" --destination "52.54,13.42"

# Switch provider with --provider
everymap --provider google --api-key $GOOGLE_KEY geocode "Brandenburg Gate, Berlin"
everymap --provider tomtom --api-key $TOMTOM_KEY geocode "Berlin"
everymap --provider mapbox --api-key $MAPBOX_KEY route --origin "52.52,13.405" --destination "52.54,13.42"
everymap --provider radar --api-key $RADAR_KEY geocode "Berlin"

# Output formats: json (default), pretty, summary
everymap --output summary --api-key $KEY geocode "Paris"

# Verbose mode shows the redacted request URL, raw body, and timing on stderr
everymap --verbose --api-key $KEY geocode "Paris"

# Binary commands save to a file (--output-file; defaults: tile.png, tile.mvt, map.png)
everymap --api-key $KEY tile --z 14 --x 4494 --y 2832
everymap --provider tomtom --api-key $TOMTOM_KEY tile --variant raster --z 14 --x 4494 --y 2832 --format png
# Vector tiles: a provider that serves MVT, e.g. HERE or MapBox
everymap --provider mapbox --api-key $MAPBOX_KEY tile --variant vector --z 14 --x 4494 --y 2832
everymap --api-key $KEY map-image --lat 52.52 --lng 13.40 --zoom 14

# Negative longitudes: use --lng=VALUE (with =) to avoid CLI arg parsing issues
everymap --api-key $KEY reverse-geocode --lat 37.77 --lng=-122.42
```

API keys can also come from the `EVERYMAP_API_KEY` environment variable or `~/.everymap/config.toml`:

```toml
[providers.here]
api_key = "your-here-key"

[providers.google]
api_key = "your-google-key"
```