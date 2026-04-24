#!/usr/bin/env bash
# Dev-friendly test runner using cargo-nextest.
# Usage:
#   ./scripts/test.sh              # all tests, default profile
#   ./scripts/test.sh --ci         # CI profile (fail-fast, JUnit XML)
#   ./scripts/test.sh --here       # only HERE provider crate
#   ./scripts/test.sh --google     # only Google provider crate
#   ./scripts/test.sh --tomtom     # only TomTom provider crate
#   ./scripts/test.sh --mapbox     # only MapBox provider crate
#   ./scripts/test.sh --radar      # only Radar provider crate
#   ./scripts/test.sh --core       # only everymap-core crate
#   ./scripts/test.sh --cli        # only everymap-cli crate
#   ./scripts/test.sh --bench      # only everymap-bench crate
#   ./scripts/test.sh --live       # include live API tests (set env vars)

set -euo pipefail

PROFILE="default"
PACKAGES=()
EXTRA_ARGS=()

for arg in "$@"; do
  case "$arg" in
    --ci)     PROFILE="ci" ;;
    --here)   PACKAGES+=("-p" "everymap-providers-here") ;;
    --google) PACKAGES+=("-p" "everymap-providers-google") ;;
    --tomtom) PACKAGES+=("-p" "everymap-providers-tomtom") ;;
    --mapbox) PACKAGES+=("-p" "everymap-providers-mapbox") ;;
    --radar)  PACKAGES+=("-p" "everymap-providers-radar") ;;
    --core)   PACKAGES+=("-p" "everymap-core") ;;
    --cli)    PACKAGES+=("-p" "everymap-cli") ;;
    --bench)  PACKAGES+=("-p" "everymap-bench") ;;
    --live)   EXTRA_ARGS+=("--ignored") ;;
    *)        EXTRA_ARGS+=("$arg") ;;
  esac
done

if ! command -v cargo-nextest &>/dev/null; then
  echo "Installing cargo-nextest..."
  cargo install cargo-nextest --locked
fi

echo "Running tests with profile: ${PROFILE}"
exec cargo nextest run --all-features --profile "${PROFILE}" "${PACKAGES[@]}" "${EXTRA_ARGS[@]}"