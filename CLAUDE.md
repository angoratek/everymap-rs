# CLAUDE.md — Project Guidance for Claude Code

## Project Overview
EveryMap-RS is a modular Rust geospatial API wrapper with provider abstraction. Currently implements HERE Technologies APIs with plans for MapBox, TomTom, and Google.

## Architecture
- **Workspace**: `everymap-core` (traits, types, auth, error) + `everymap-providers-here` (HERE implementations)
- **Pattern**: Each domain trait in `everymap-core` has associated `Options` and `Response` types. Provider implementations define their own rich types and map to the core trait.
- **Each domain module** has its own `types.rs` submodule with full HERE-specific request/response types.
- **Explicit re-exports** in `domain/mod.rs` and `lib.rs` to avoid glob conflicts between modules.

## Key Conventions
- Use `async-trait` for all domain traits.
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding.
- Use `wiremock` for contract tests — mock HERE API responses and validate deserialization.
- Each domain has a `const BASE_URL` for its API endpoint.
- `HereClient` is a thin HTTP + auth wrapper; domains construct their own full URLs.
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`.
- All optional response fields use `#[serde(default)]`.
- Helper functions (`add_option`, `add_option_ref`) for building query params from optional fields.
- Binary responses (tiling, imaging) use `response.bytes().await?.to_vec()` for body and `response.headers()` for content-type.
- Provider-specific rich types live in `types.rs` submodules within each domain.
- Core traits return simplified types; providers expose rich HERE-specific methods alongside the trait impl.

## Build & Test Commands
- `cargo clippy -- -D warnings` — must pass with zero warnings
- `cargo test` — runs all 31 contract tests
- `cargo build` — verify compilation

## Domain Module Layout
Each domain follows this pattern:
```
src/domain/
  ├── mod.rs          # Module declarations + explicit re-exports
  ├── search.rs       # HereGeocoder implementation
  ├── search/types.rs # HereSearchItem, HereAddress, etc.
  ├── routing.rs      # HereRouter implementation
  ├── routing/types.rs
  ... (one per domain)
```

## Base URLs
- Search: `https://geocode.search.hereapi.com/v1`
- Routing: `https://router.hereapi.com/v8`
- Isoline: `https://isoline.router.hereapi.com/v8`
- Matching: `https://routematching.hereapi.com/v8`
- Tour: `https://tourplanning.hereapi.com/v3`
- Traffic: `https://data.traffic.hereapi.com/v7`
- Tiling: `https://vector.hereapi.com/v2`
- Positioning: `https://positioning.hereapi.com/v2`
- Attributes: `https://smap.hereapi.com/v8`
- Imaging: `https://image.maps.hereapi.com/mia/v3`

## Testing Pattern
- Test files: `everymap-providers-here/tests/{domain}_contract.rs`
- Each test creates a `MockServer`, defines expected JSON responses, verifies deserialization and trait behavior.
- Tests use `with_base_url()` to point at the mock server.
- Auth: `Arc::new(ApiKeyProvider::new("test-key".to_string(), "apiKey".to_string()))`

## Git Rules
- **Never commit without explicit user approval.** Always ask before committing. Do not assume the user wants a commit after making changes.
- Do not push to remote unless explicitly asked.

## What NOT to Do
- Don't add `dyn` dispatch where generics suffice (zero-cost abstractions).
- Don't leak HERE-specific types into `everymap-core`.
- Don't use glob re-exports (`pub use module::*`) in `domain/mod.rs` — use explicit re-exports to avoid conflicts.
- Don't add dependencies without workspace-level coordination.
- Don't skip the `#[serde(default)]` on optional response fields.