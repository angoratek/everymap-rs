# One contract surface

**Status:** design, owner ruling 2026-11-05. Work: `PLAN.md` M4.
**Question it answers:** how does the provider set present itself when some
crates were written before the typed contracts and some after?

## Outcome

A reader — a person or an agent — sees one provider matrix, one vocabulary and
one set of traits. Every provider implements the newest contract for each
capability its service offers, and no legacy trait survives to be special-cased
in the docs, the CLI or the benchmark.

## The rule

1. **One generation of contracts.** A capability has exactly one trait:
   `Geocoder`, `PlaceSearchProvider`, `AutocompleteProvider`, `Router`,
   `DistanceMatrixProvider`, `TrafficProvider`, `RasterTileProvider`,
   `VectorTileProvider`, `ElevationProvider`, `IsolineProvider`,
   `RouteMatcher`, `TourPlanner`, `NetworkPositioner`, `AttributeProvider`,
   `MapImageProvider`, `OfflineRoutingProvider`.
2. **Contract vocabulary.** Documents name the contract, not the old domain
   word: Geocoding (not Search), Raster tiles (not Tiling), Vector tiles,
   Place search, Type-ahead, Distance matrix, and the rest as written above.
   The trait is named beside it.
3. **A provider serves what its service offers.** A gap is a `—` in the
   matrix, never a second trait. Where a service can only answer a narrower
   shape — a two-endpoint router, a raster-only tileset — the crate's README
   and the matrix note state it, and the trait's default behaviour carries it.
4. **No legacy trait.** The untyped `TileProvider` (string format, resolution
   in provider extras) is removed rather than kept beside the typed contracts.

## Audit (2026-11-05)

What each crate actually implements; the gaps are the matrix's `—`.

| Contract | HERE | Google | TomTom | MapBox | Radar | MapQuest | Photon | OSRM | Valhalla | URL-template |
|---|---|---|---|---|---|---|---|---|---|---|
| Geocoding | yes | yes | yes | yes | yes | yes | yes | — | — | — |
| Place search | — | — | — | — | — | yes | yes | — | — | — |
| Type-ahead | — | — | — | — | — | yes | yes | — | — | — |
| Routing | yes (two endpoints) | yes (two endpoints) | yes (two endpoints) | yes (two endpoints) | yes (two endpoints) | yes (waypoints) | — | yes (waypoints) | yes (waypoints) | — |
| Distance matrix | — | — | — | — | — | yes | — | — | — | — |
| Traffic | yes | — | yes | — | — | yes | — | — | — | — |
| Raster tiles | — | — | yes (migrate) | — | — | yes | — | — | — | raster crate |
| Vector tiles | yes (migrate) | — | — | yes (migrate) | — | — | — | — | — | vector crate |
| Elevation | — | — | — | — | — | — | — | — | — | — |
| Isoline | yes | — | yes | yes | — | — | — | — | — | — |
| Matching | yes | yes | yes | yes | yes | — | — | — | — | — |
| Tour | yes | — | yes | yes | yes | — | — | — | — | — |
| Positioning | yes | yes | — | — | — | — | — | — | — | — |
| Attributes | yes | yes | — | — | — | — | — | — | — | — |
| Imaging | yes | yes | yes | yes | — | — | — | — | — | — |

## The three tile migrations

- **HERE** serves the Vector Tile API v2, whose only output segment is `omv`;
  the service refuses `pbf` with `E603400`. The `omv` bytes are Mapbox Vector
  Tile protobuf — a fetched tile decodes as an MVT v2 layer named `earth` with
  extent 4096 — so the crate moved to `VectorTileProvider` requesting `omv`,
  with a descriptor naming the tileset (`here/vector`), the attribution and the
  documented 0–17 span. A caller that wants another layer reaches it through
  `fetch_vector_tile_in_layer`. Both facts were verified against the live
  service on 2026-11-05: the `pbf` refusal and the `Zoom level 18 is invalid.
  Accepted zoom levels are: [0-17]` answer.
- **MapBox** serves the Vector Tiles API v4 (`/v4/{tileset}/{z}/{x}/{y}.mvt`).
  The crate moves to `VectorTileProvider`; the tileset id becomes the
  descriptor's source name (`mapbox/{tileset}`), with MapBox attribution and
  the tileset's zoom span.
- **TomTom** serves the Map Display API raster (`/map/1/tile/{layer}/{style}/…`).
  The crate moves to `RasterTileProvider`: the typed request carries the
  format (PNG) and resolution (`@2x` is not served, so Retina is refused with a
  validation error), the layer and style stay as provider extras.

Verification status: the HERE facts above were checked against the live service.
The MapBox and TomTom endpoints follow their documented URL shapes but were not
exercised live in this environment, which carries no credential for either; the
contract tests pin the request shape and a live session should confirm the
answers before the next release.

## The CLI and the benchmark

Both tools consume the same traits. The `tile` command becomes
`--variant raster|vector` with the typed request's `--format` and
`--resolution` for raster, and the bench's tile scenario measures the same two
contracts. A provider that does not serve tiles answers with the unsupported
error, as with any other gap.

## Documentation

- One matrix in `README.md`, rows by contract, one column per crate.
- Each crate's README lists the contracts it implements and the ones it does
  not, in the same words.
- `everymap-core`'s crate description names the provider family, so the
  crates.io page says what the workspace is.
- The "Adding a provider" section states the contract-first rule: a new
  capability is a new contract for every provider, never a second trait for
  one of them.
