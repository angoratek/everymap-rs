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
| Place search | `PlaceSearchProvider` | `PhotonPlaceSearchProvider`, settled search defaulting to ten results |
| Type-ahead | `AutocompleteProvider` | `PhotonAutocompleteProvider`, type-ahead defaulting to five results |

Not served: routing, distance matrix, traffic, raster tiles, vector tiles, elevation, isoline, matching, tour, positioning, attributes, imaging and offline routing. Those calls return `EveryMapError::UnsupportedDomain`.

Feature classes map onto the core's `SearchQuality` scale (`house`/`building` →
street address, `street`, `locality`/`district`/`city`/`town`/`village`,
`county`, `state`, `country`, `postcode`; any other class is a point), and
each feature's address parts fill the core `Address` record.

`PhotonClient` carries the shared pipeline setters — `set_observer`,
`set_rate_limiter`, `set_retry_policy`, `set_request_timeout` and
`with_transport` — so a deployment's budget and diagnostics match every other
provider. The geocoder sends a reverse language as `lang`, honours the reverse
result limit, and refuses `country_codes` and a reverse-geocode radius with a
validation error instead of dropping them.

## Targets

The crate builds for `wasm32-unknown-unknown` as well as native targets; every
request goes through `everymap-core`'s shared client, which selects the
platform transport.
