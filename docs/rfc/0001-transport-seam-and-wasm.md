# RFC 0001: Transport Seam and WebAssembly Target

- Status: accepted for implementation
- Scope: `everymap-core`, provider clients, CI
- Compatibility: additive for the domain traits; the provider-client construction surface gains a backend-neutral form

## Summary

`everymap-core` currently leaks `reqwest` through its public client surface
(`AuthProvider::apply(RequestBuilder)`, `ProviderClient::request(RequestBuilder)`)
and depends on `tokio` with its full feature set. That makes the core crate
unusable from `wasm32-unknown-unknown`, where `tokio`'s networking runtime and
`mio` do not compile. This RFC introduces a backend-agnostic HTTP seam, a
browser-fetch transport for WebAssembly, and shared client reliability
features: token-bucket rate limiting, idempotent-GET retry, per-request
timeouts, and a redacting request observer.

The domain traits (`Geocoder`, `Router`, `TileProvider`, and the rest) are not
changed by this RFC. A later RFC covers the navigation-grade domain additions.

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

## Planned domain additions (later phases)

These are recorded here so the transport seam is sized for them, but they are
not implemented in this RFC:

| Domain | Planned shape |
|---|---|
| Routing | Navigation-grade result: legs, maneuvers with speech and lane guidance, shape geometry, and per-route capabilities |
| Raster tiles | Typed `RasterTileProvider` with tile size and image format instead of `format: String` plus `provider_extra` |
| Vector tiles | `VectorTileProvider` returning a source descriptor plus tile bytes |
| Traffic | Bounding-box flow queries with geometry, not center-plus-radius only |
| Search | Autocomplete as a first-class trait with typed suggestions |
| Routing | Origin/destination matrix as a first-class trait |
| Elevation | Elevation lookup trait |
| Offline routing | Offline graph trait, independent of network providers |

Each addition is a new trait or a new method on an existing trait, added in a
way that does not break existing implementations.

## Consequences

- `everymap-core` builds and clippy-checks for `wasm32-unknown-unknown`.
- `reqwest` remains an implementation detail of the native transport; it is no
  longer part of the core public API.
- The provider crates change only mechanically: `reqwest::Method` becomes
  `everymap_core::http::Method`, and response bodies are read from
  `HttpResponse` instead of `reqwest::Response`.
- `EveryMapError::ClientError` stays native-only; WebAssembly transport
  failures map to the new `EveryMapError::TransportError`.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo clippy -p everymap-core --target wasm32-unknown-unknown --lib -- -D warnings
cargo build -p everymap-core --target wasm32-unknown-unknown
```

The last two commands become a `wasm` job in `.github/workflows/ci.yml`.
