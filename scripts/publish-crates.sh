#!/usr/bin/env bash
set -euo pipefail

# publish-crates.sh
# Publishes workspace crates to crates.io in dependency order.
#
# Usage: ./scripts/publish-crates.sh [--dry-run]
#   --dry-run    Run cargo publish --dry-run for each crate without publishing

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Publish order: dependencies first, consumers last
CRATES=(
    everymap-core
    everymap-providers-here
    everymap-providers-google
    everymap-providers-tomtom
    everymap-providers-mapbox
    everymap-providers-radar
    everymap-cli
)

PUBLISH_FLAGS=()
$DRY_RUN && PUBLISH_FLAGS+=(--dry-run --allow-dirty)

# First publish needs --no-verify because core's dev-deps aren't on crates.io yet
FIRST_PUBLISH=true

FAILED=0

for crate in "${CRATES[@]}"; do
    # Skip publish = false crates
    if grep -q 'publish\s*=\s*false' "$REPO_ROOT/$crate/Cargo.toml" 2>/dev/null; then
        echo "SKIP $crate (publish = false)"
        continue
    fi

    echo "=== Publishing $crate ==="
    EXTRA_FLAGS=()
    # Skip verification for first crate (core has dev-deps not yet on crates.io)
    if $FIRST_PUBLISH; then
        EXTRA_FLAGS+=(--no-verify)
        FIRST_PUBLISH=false
    fi
    if cargo publish --manifest-path "$REPO_ROOT/$crate/Cargo.toml" "${PUBLISH_FLAGS[@]}" "${EXTRA_FLAGS[@]}" 2>&1; then
        echo "OK   $crate"
        # Wait for crates.io to index before dependent crates can find it
        if ! $DRY_RUN; then
            echo "Waiting 30s for crates.io to index $crate..."
            sleep 30
        fi
    else
        echo "FAIL $crate"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
if [[ $FAILED -eq 0 ]]; then
    echo "All crates published successfully."
else
    echo "$FAILED crate(s) failed to publish."
    exit 1
fi