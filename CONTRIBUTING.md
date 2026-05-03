# Contributing to EveryMap-RS

Thank you for your interest in contributing! This guide covers the basics.

## Development Setup

1. **Rust toolchain**: Install via [rustup](https://rustup.rs/). Minimum supported version: 1.75.
2. **Clone and build**:
   ```bash
   git clone https://github.com/angoratek/everymap-rs.git
   cd everymap-rs
   cargo build
   ```
3. **Run tests**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

## Project Structure

```
everymap-rs/
├── everymap-core/          # Traits, types, auth, error, client
├── everymap-providers-here/   # HERE Technologies
├── everymap-providers-google/ # Google Maps
├── everymap-providers-tomtom/ # TomTom
├── everymap-providers-mapbox/ # MapBox
├── everymap-providers-radar/  # Radar
├── everymap-cli/           # CLI tool (19 commands)
└── everymap-bench/         # Benchmark framework
```

## Adding a New Provider

1. Create `everymap-providers-{name}/` crate depending on `everymap-core`
2. Create `client.rs` — thin wrapper around `ProviderClient` (see `GoogleClient` as template)
3. Define `const PROVIDER_NAME: &str = "name"` in the client module
4. Create `domain/geo.rs` — shared lat/lng type
5. Implement supported domain traits (start with `Geocoder` + `Router`)
6. Add `From<ProviderX> for CoreType` conversions for all response types
7. Add stub implementations using `everymap_core::unsupported_*!` macros
8. Add provider to `ProviderRegistry` in `everymap-cli/src/provider.rs`
9. Add provider section in `everymap-cli/src/config.rs`
10. Add workspace member in root `Cargo.toml`
11. Add provider to `everymap-bench/src/benchmark.rs`

## Coding Conventions

- Use `async-trait` for all domain traits
- Use `flexpolyline` crate (v1.0) for HERE Flexible Polyline encoding/decoding
- Use `wiremock` for contract tests — mock API responses and validate deserialization
- Each domain has a `const BASE_URL` for its API endpoint
- Provider client wrappers delegate to `everymap_core::client::ProviderClient`
- All enums use `#[serde(rename_all = "snake_case")]` or explicit `#[serde(rename = "...")]`
- All optional response fields use `#[serde(default)]`
- Binary responses (tiling, imaging) use `response.bytes().await?.to_vec()` for body
- Provider-specific rich types live in `types.rs` submodules
- Core traits return enriched types; providers expose rich methods alongside trait impls
- Don't leak provider-specific types into `everymap-core`
- Don't use glob re-exports in `domain/mod.rs` for non-types modules
- Use full, descriptive variable names (`config` not `cfg`, `options` not `opts`, etc.)
- API keys are zeroized on drop via `zeroize` crate

## Testing

- **Unit tests**: Inline `#[cfg(test)] mod tests` in each module
- **Contract tests**: `tests/{domain}_contract.rs` using `wiremock` for each provider
- **CLI integration tests**: `everymap-cli/tests/` for CLI output validation
- **Live API tests**: `tests/live_api.rs` gated behind `#[ignore]` and `integration` feature

See [TESTING.md](TESTING.md) for the full testing guide.

## Pull Request Process

1. Ensure `cargo clippy -- -D warnings` passes with zero warnings
2. Ensure `cargo test` passes (all 740+ tests)
3. Ensure `cargo fmt --all -- --check` passes
4. Add tests for any new functionality
5. Update documentation (CLAUDE.md, README.md, TESTING.md) as needed
6. Keep PRs focused — one feature or fix per PR

## Release Process

- All crates share a workspace version in `Cargo.toml`
- Run `./scripts/check-publish-readiness.sh --verbose` before releasing
- Tag releases with `v0.x.0` format