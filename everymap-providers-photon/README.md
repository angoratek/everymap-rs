# EveryMap-RS

One trait surface over eleven geospatial providers for Rust: **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, **Radar**, **MapQuest**, **Photon**, **OSRM**, **Valhalla**, and URL-template sources for any `{z}/{x}/{y}` raster or vector tileset. Business logic targets the contract, and a provider is a constructor change rather than a rewrite. Every provider crate builds for native and browser targets.

Contracts: geocoding and reverse geocoding, settled place search, type-ahead completion, navigation-grade routing, distance matrices, traffic flow and incidents, raster tiles, vector tiles, isolines, map matching, tour planning, network positioning, geodata attributes, static map images, elevation and offline routing.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-providers-mapquest`](https://crates.io/crates/everymap-providers-mapquest) · [`everymap-providers-photon`](https://crates.io/crates/everymap-providers-photon) · [`everymap-providers-osrm`](https://crates.io/crates/everymap-providers-osrm) · [`everymap-providers-valhalla`](https://crates.io/crates/everymap-providers-valhalla) · [`everymap-providers-url-template-raster`](https://crates.io/crates/everymap-providers-url-template-raster) · [`everymap-providers-url-template-vector`](https://crates.io/crates/everymap-providers-url-template-vector) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

---

# everymap-providers-photon

Photon geocoding for [EveryMap-RS](https://github.com/angoratek/everymap-rs):
forward and reverse geocoding, settled place search and type-ahead over the
public [Photon](https://photon.komoot.io) instance or any self-hosted
deployment.

Photon is keyless. Point the client at a deployment root and pick the contract
you need:

```rust,no_run
use std::sync::Arc;
use everymap_core::domains::search::{Geocoder, GeocodeOptions};
use everymap_providers_photon::{PhotonClient, PhotonGeocoder, PhotonProviderConfiguration};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = Arc::new(PhotonClient::new(PhotonProviderConfiguration::default()));
let geocoder = PhotonGeocoder::new(client);
let response = geocoder.geocode("Brandenburg Gate, Berlin", &GeocodeOptions::default()).await?;
# let _ = response;
# Ok(())
# }
```

## Contracts

| Contract | Trait | Implementation |
|---|---|---|
| Geocoding | `Geocoder` | `PhotonGeocoder`, forward and reverse queries over `/api` and `/reverse` |
| Place search | `PlaceSearchProvider` | `PhotonPlaceSearchProvider`, settled search with a ten-result cap |
| Type-ahead | `AutocompleteProvider` | `PhotonAutocompleteProvider`, type-ahead with a five-result cap |

Not served: routing, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

Feature classes map onto the core's `SearchQuality` scale (`house` →
street address, `street`, `locality`/`city`, `county`, `state`, `country`,
`postcode`), and each feature's address parts fill the core `Address` record.

## Targets

The crate builds for `wasm32-unknown-unknown` as well as native targets; every
request goes through `everymap-core`'s shared client, which selects the
platform transport.
