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
