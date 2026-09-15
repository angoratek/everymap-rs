# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-cli

Unified command-line interface for [EveryMap-RS](https://github.com/angoratek/everymap-rs) — one binary, 11 geospatial commands, 5 providers (**HERE**, **Google**, **TomTom**, **MapBox**, **Radar**).

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
| `tile` | Download a map tile to a file |
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

# Binary commands save to a file (--output-file; defaults: tile.omv, map.png)
everymap --api-key $KEY tile --z 14 --x 4494 --y 2832
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