#!/usr/bin/env bash
set -euo pipefail

# browser-smoke.sh
#
# Builds the browser-smoke crate for wasm32, generates the wasm-bindgen web
# module, and runs the Playwright harness.
#
# Requirements: the wasm32-unknown-unknown target, `wasm-bindgen` whose version
# matches the crate's resolved wasm-bindgen, and Playwright with Chromium.
#
# Usage: scripts/browser-smoke.sh

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

wasm_bindgen_version="$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep 'version' | head -1 | cut -d'"' -f2)"
installed_version="$(wasm-bindgen --version | awk '{print $2}')"
if [[ "$installed_version" != "$wasm_bindgen_version" ]]; then
    printf 'browser-smoke: wasm-bindgen %s is installed, the workspace resolves %s\n' \
        "$installed_version" "$wasm_bindgen_version" >&2
    printf 'install it with: cargo install wasm-bindgen-cli --version %s\n' \
        "$wasm_bindgen_version" >&2
    exit 1
fi

target_directory="${CARGO_TARGET_DIR:-target}"
cargo build -p everymap-browser-smoke --target wasm32-unknown-unknown
wasm-bindgen \
    "$target_directory/wasm32-unknown-unknown/debug/everymap_browser_smoke.wasm" \
    --out-dir everymap-browser-smoke/pkg \
    --target web

node scripts/browser-smoke.mjs
