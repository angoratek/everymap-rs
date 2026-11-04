# RFC 0001: Transport Seam and WebAssembly Target

- Status: accepted for implementation
- Scope: `everymap-core`, provider clients, CI
- Compatibility: additive for the domain traits; the provider-client construction surface gains a backend-neutral form. Phase 2 adds serde-defaulted fields to `RouteResult`, `RouteStep` and `RouteOptions`, which is a mechanical source migration for struct literals.

## Summary

`everymap-core` currently leaks `reqwest` through its public client surface
(`AuthProvider::apply(RequestBuilder)`, `ProviderClient::request(RequestBuilder)`)
and depends on `tokio` with its full feature set. That makes the core crate
unusable from `wasm32-unknown-unknown`, where `tokio`'s networking runtime and
`mio` do not compile. This RFC introduces a backend-agnostic HTTP seam, a
browser-fetch transport for WebAssembly, and shared client reliability
features: token-bucket rate limiting, idempotent-GET retry, per-request
timeouts, and a redacting request observer.

The domain traits (`Geocoder`, `Router`, `TileProvider`, and the rest) were not
changed by the transport work. Phase 2 adds navigation-grade routing and typed
raster and vector tile contracts without removing the original surfaces; those
decisions are recorded in [Phase 2](#phase-2-navigation-grade-routing-and-typed-tiles).

## Decisions

### 1. One transport trait, plain request and response values

```rust
pub struct HttpRequest {
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<HttpBody>,
    timeout: Option<Duration>,
}

pub struct HttpResponse {
    status: u16,
    headers: HttpHeaders,
    body: Vec<u8>,
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse>;
}
```

`HttpRequest` carries owned data only. It is `Clone`, so a retry can resend the
same request without rebuilding provider code. Construction and access go
through methods (`new`, `get`, `post`, `method`, `url_with_query`, `headers`,
`body`, `timeout_value`). `HttpResponse` exposes status, case-insensitive
headers, and bytes; JSON decoding stays in the client.

`HttpRequest` mirrors the small subset of `RequestBuilder` the providers use:
`query`, `header`, `json`, `form`, `raw_body`, and `timeout`. Query pairs are
encoded by the request, not by the backend.

### 2. Native and WebAssembly implementations

- `ReqwestTransport` (`cfg(not(target_arch = "wasm32")))` wraps
  `reqwest::Client`. It preserves the current native behaviour, including
  `ProviderClient::with_client_builder` for callers that need proxy or TLS
  configuration.
- `FetchTransport` (`cfg(target_arch = "wasm32"))` builds a `web_sys::Request`
  and calls `window.fetch`. Because `wasm-bindgen-futures` futures are not
  `Send`, the fetch runs on `wasm_bindgen_futures::spawn_local` and returns its
  result through a `futures_channel::oneshot`, keeping `HttpTransport::send`
  `Send` and therefore usable from the existing `Send` async traits.
- `default_transport()` selects the implementation for the target. Callers may
  inject any `Arc<dyn HttpTransport>`, which is also the testing seam.

### 3. Authentication without transport types

`AuthProvider::apply` takes and returns an `HttpRequest`. API-key providers add
a query parameter; header providers validate and add a header; OAuth 2.0
fetches and caches its token through an `Arc<dyn HttpTransport>` held by the
provider. No auth implementation names a `reqwest` type.

### 4. Target-gated dependencies

`reqwest` and the native `tokio` feature set move to
`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. The WebAssembly
target gets `tokio` with only the `sync` feature, plus `wasm-bindgen`,
`wasm-bindgen-futures`, `js-sys`, `web-sys`, and `futures-channel`. No
user-facing feature flag is required: the target selects the backend.

### 5. Shared client reliability

All three behaviours are opt-out additions to `ProviderClient`:

- **Token bucket** — `TokenBucket::new(capacity, refill_per_second)` gates
  requests. Disabled unless configured, so existing callers see no pacing.
- **Idempotent-GET retry** — `RetryPolicy` (default: 3 attempts, 100 ms
  exponential base, 2 s cap) retries `GET` and `HEAD` on transport errors,
  `5xx`, and `429` responses whose `Retry-After` is at most 5 seconds.
  Non-idempotent methods are never retried. A long `Retry-After` is returned to
  the caller instead of sleeping.
- **Timeouts** — a 30-second default per attempt, configurable per client and
  overridable per request.

### 6. Redacting request observer

`RequestObserver` receives a `RequestObservation` containing the provider,
method, attempt, and a **redacted** URL, and a `ResponseObservation` with
status and elapsed time. Credential-bearing query values (`apiKey`, `key`,
`access_token`, `token`, `client_secret`, `api_key`, `subscription-key`,
`auth`) are replaced with `***` before the observer is called. Header values
are never included; sensitive header names are listed by name only.

Observers must not receive request or response bodies. The client-side
`redact_url` helper is public so custom logging can use the same rule.

## Phase 2: navigation-grade routing and typed tiles

These decisions implement the routing and tile rows of the adoption note's
phase 2. The remaining phase-2 domains (bounding-box traffic with geometry,
autocomplete, distance matrix, elevation and offline routing) are unchanged by
this change and stay open.

### 7. Ordered waypoints are a new method, not a changed signature

`Router::calculate_route(start, end, options)` keeps its signature. The
navigation-grade entry point is a second method:

```rust
async fn calculate_route_with_waypoints(
    &self,
    waypoints: &[Coordinate],
    options: &RouteOptions,
) -> EveryMapResult<RouteResponse>;
```

Its default body delegates a two-waypoint request to `calculate_route` and
reports `EveryMapError::UnsupportedDomain` with the exported
`INTERMEDIATE_WAYPOINTS_DOMAIN` for a longer request. Fewer than two waypoints
is `EveryMapError::ValidationError`. This keeps every existing provider
compiling while making the capability gap explicit instead of silently
dropping intermediate stops; phase 3 providers override the method. A new
`Router::route_capabilities()` defaults to `RouteCapabilities::all_supported()`
and follows the same opt-in override rule.

Rationale: the SDK calls an ordered-waypoint method, but changing the existing
signature would break five provider crates at once for a capability they do not
have yet. The default is a truthful two-endpoint adapter, not a silent success.

### 8. Navigation records extend the existing route types

- `RouteResult` gains `legs: Vec<RouteLeg>`; `RouteLeg` carries `distance`,
  `duration`, `steps` and a `shape` polyline. The original flat `steps` list
  and `geometry` polyline stay, so a provider that reports only the
  two-endpoint projection remains valid and leaves `legs` empty.
- `RouteStep` gains `maneuver_type`, `speech_instruction`, `lane_guidances`
  and `shape`. All four are `#[serde(default)]`, so a payload recorded before
  the fields existed still deserializes.
- `ManeuverType` covers departure, arrival, continue, turns, merge, on-ramp,
  off-ramp, fork, roundabout (with the one-based exit index) and ferry. It
  serializes as one stable kebab-case label (`departure`, `turn-left`,
  `roundabout-2`) and deserializes only from that spelling; no label is
  guessed. `TurnDirection` names the nine directions, and `LaneGuidance`
  carries a zero-based lane index, its permitted directions and whether it is
  recommended.
- `RouteStrategy` (`fastest` or `shortest`) is added to `RouteOptions`.
  `RouteCapabilities` reports the supported transport modes and whether the
  shortest strategy can be honoured, which is what lets a surface disable a
  choice it cannot serve.

Adding fields to public structs breaks exhaustive struct literals. That
migration is mechanical — the five existing provider crates add the
serde-defaulted fields in this change — and no wire format breaks. Renaming the
existing surfaces was rejected because the adoption plan assumes additive
traits with the older names kept.

### 9. Typed raster tiles

```rust
pub struct RasterTileRequest {
    pub tile_x: u32,
    pub tile_y: u32,
    pub zoom_level: u8,
    pub image_format: RasterImageFormat,       // Png | Jpeg
    pub tile_resolution: RasterTileResolution, // Standard | Retina
}

#[async_trait]
pub trait RasterTileProvider: Send + Sync {
    async fn fetch_raster_tile(&self, request: &RasterTileRequest)
        -> EveryMapResult<RasterTileData>;
}
```

`RasterTileData` echoes the request's coordinates, format and resolution
alongside the bytes (`RasterTileData::for_request` builds it in one step), so a
cache key or renderer never re-derives what the payload is. A provider that
cannot serve a format or resolution returns an error rather than silently
re-encoding. The original `TileProvider`/`TileResponse` surface stays untouched
for source compatibility; phase 3 providers implement the typed trait.

### 10. Vector tiles with a source descriptor

```rust
pub struct VectorTileSourceDescriptor {
    pub source_name: String,
    pub attribution_text: String,
    pub minimum_zoom_level: u8,
    pub maximum_zoom_level: u8,
    pub tile_encoding: VectorTileEncoding, // MapboxVectorTile | GzippedMapboxVectorTile
    pub style_hints: Vec<String>,
}

#[async_trait]
pub trait VectorTileProvider: Send + Sync {
    fn source_descriptor(&self) -> VectorTileSourceDescriptor;
    async fn fetch_vector_tile(&self, tile_x: u32, tile_y: u32, zoom_level: u8)
        -> EveryMapResult<VectorTileData>;
}
```

The descriptor carries the five facts the SDK's `geospatial-traits` and its
provider registry already rely on: a cache namespace, an attribution line, the
served zoom span and the wire encoding. `style_hints` is an addition: the
registry declares a style family and named styles per vector provider, and
carrying them on the descriptor lets a renderer choose without provider
knowledge. The field is serde-defaulted, so an SDK adapter that does not need
it can ignore it.

`VectorTileData::from_wire_bytes` classifies the payload from its leading gzip
magic number and rejects an empty payload as a `ValidationError`; an empty
response is never treated as an empty tile. The bytes stay exactly as
delivered, because decoding belongs to the renderer.

### 11. Fixtures and contract tests

New contracts are tested without a network: unit tests cover every serde
default and stable label, and integration tests drive the traits through an
in-process `HttpTransport` fake. Committed fixtures live under
`everymap-core/tests/fixtures/` and stay small: a 66-byte 1x1 PNG, a 5-byte MVT
protobuf, its 25-byte gzip envelope, and a two-leg navigation route JSON.

## Remaining planned domain additions

These are recorded so later phases are sized against the same seam; they are
not implemented yet.

| Domain | Planned shape | Status |
|---|---|---|
| Routing | Navigation-grade result: legs, maneuvers with speech and lane guidance, shape geometry, and per-route capabilities | Implemented in phase 2 |
| Raster tiles | Typed `RasterTileProvider` with tile size and image format instead of `format: String` plus `provider_extra` | Implemented in phase 2 |
| Vector tiles | `VectorTileProvider` returning a source descriptor plus tile bytes | Implemented in phase 2 |
| Traffic | Bounding-box flow queries with geometry, not center-plus-radius only | Open |
| Search | Autocomplete as a first-class trait with typed suggestions | Open |
| Routing | Origin/destination matrix as a first-class trait | Open |
| Elevation | Elevation lookup trait | Open |
| Offline routing | Offline graph trait, independent of network providers | Open |

Each addition is a new trait or a new method on an existing trait, added in a
way that does not break existing implementations. The one deliberate exception
is the serde-defaulted field extensions in decision 8, whose migration is
mechanical.

## Phase status

The phases are the adoption note's; status is recorded against the exact tree
that carries this change.

| Phase | Scope | Status |
|---|---|---|
| 0 | Design and RFC, plus the adoption note in the SDK repository | Complete |
| 1 | Core plumbing: transport seam, browser-fetch wasm transport, feature-gated tokio, wasm CI, pacing, retry, timeouts, redacting observer | Complete (`feat/transport-and-wasm`, draft PR #22) |
| 2 | Core domains: navigation-grade routing and typed raster/vector tiles | This change; traffic geometry, autocomplete, matrix, elevation and offline routing still open |
| 3 | Providers: MapQuest, OSRM, Valhalla, URL-template raster and vector, OSM US/OpenFreeMap descriptors, with recorded fixtures and contract tests | Not started |
| 4 | Release 0.3.0 with the wasm/native CI matrix and the changelog | Not started |
| 5 | Switch the SDK: registry, catalog, cache wrappers, native registration and wasm dispatch | Not started |
| 6 | Docs and gates: provider-genericity guide, crate layout, name guard, full platform gates | Not started |

## Consequences

- `everymap-core` builds and clippy-checks for `wasm32-unknown-unknown`.
- `reqwest` remains an implementation detail of the native transport; it is no
  longer part of the core public API.
- The provider crates change only mechanically: `reqwest::Method` becomes
  `everymap_core::http::Method`, and response bodies are read from
  `HttpResponse` instead of `reqwest::Response`.
- `EveryMapError::ClientError` stays native-only; WebAssembly transport
  failures map to the new `EveryMapError::TransportError`.
- The original `TileProvider` and two-endpoint `Router` surfaces remain
  available. Phase 3 providers implement the typed tile traits and override
  `calculate_route_with_waypoints`; no consumer is forced to switch at once.
- The navigation and tile records are plain owned values, so they build for
  every target that builds the core, WebAssembly included.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p everymap-core --target wasm32-unknown-unknown
cargo clippy -p everymap-core --target wasm32-unknown-unknown --lib -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
bash scripts/generate-readmes.sh --check
```

The wasm commands run as the `wasm` job in `.github/workflows/ci.yml`. The
contract tests read only committed fixtures and never open a socket.
