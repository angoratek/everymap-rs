# EveryMap-RS

A modular, type-safe Rust wrapper for geospatial APIs across **HERE Technologies**, **Google Maps**, **TomTom**, **MapBox**, and **Radar**. EveryMap-RS uses domain-driven design: 10 geospatial capabilities (search, routing, isolines, map matching, tour planning, traffic, tiling, positioning, attributes, and imaging) are defined as traits in `everymap-core` and implemented by per-provider crates — switch providers by changing one line of code.

- Repository: <https://github.com/angoratek/everymap-rs>
- Core API docs: <https://docs.rs/everymap-core>
- Other crates: [`everymap-core`](https://crates.io/crates/everymap-core) · [`everymap-providers-here`](https://crates.io/crates/everymap-providers-here) · [`everymap-providers-google`](https://crates.io/crates/everymap-providers-google) · [`everymap-providers-tomtom`](https://crates.io/crates/everymap-providers-tomtom) · [`everymap-providers-mapbox`](https://crates.io/crates/everymap-providers-mapbox) · [`everymap-providers-radar`](https://crates.io/crates/everymap-providers-radar) · [`everymap-cli`](https://crates.io/crates/everymap-cli)

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

| Contract | Implementation |
|---|---|
| `Geocoder` | `PhotonGeocoder` — forward and reverse queries over `/api` and `/reverse` |
| `PlaceSearchProvider` | `PhotonPlaceSearchProvider` — settled search with a ten-result cap |
| `AutocompleteProvider` | `PhotonAutocompleteProvider` — type-ahead with a five-result cap |

Feature classes map onto the core's `SearchQuality` scale (`house` →
street address, `street`, `locality`/`city`, `county`, `state`, `country`,
`postcode`), and each feature's address parts fill the core `Address` record.

## Targets

The crate builds for `wasm32-unknown-unknown` as well as native targets; every
request goes through `everymap-core`'s shared client, which selects the
platform transport.
