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

PUBLISH_FLAGS=(--locked)
$DRY_RUN && PUBLISH_FLAGS+=(--dry-run --allow-dirty)

FAILED=0

# Wait until crates.io has indexed the crate (dependent crates need it in
# the registry index before they can publish). Polls for up to ~2 minutes.
wait_for_index() {
    local crate_name="$1"
    local attempts=0
    until curl -fsSL -H "User-Agent: everymap-rs-release (github.com/angoratek/everymap-rs)" \
            "https://crates.io/api/v1/crates/${crate_name}" > /dev/null 2>&1; do
        attempts=$((attempts + 1))
        if [[ $attempts -ge 24 ]]; then
            echo "WARN  ${crate_name} not indexed after ~2 minutes; continuing anyway"
            return 0
        fi
        sleep 5
    done
    echo "Indexed ${crate_name} (waited $((attempts * 5))s)"
}

for crate in "${CRATES[@]}"; do
    # Skip publish = false crates
    if grep -q 'publish\s*=\s*false' "$REPO_ROOT/$crate/Cargo.toml" 2>/dev/null; then
        echo "SKIP $crate (publish = false)"
        continue
    fi

    echo "=== Publishing $crate ==="
    publish_status=0
    publish_output=$(cargo publish --manifest-path "$REPO_ROOT/$crate/Cargo.toml" "${PUBLISH_FLAGS[@]}" 2>&1) || publish_status=$?
    if [[ $publish_status -eq 0 ]]; then
        echo "OK   $crate"
        if ! $DRY_RUN; then
            wait_for_index "$crate"
        fi
    elif $DRY_RUN && grep -q "no matching package named \`everymap-" <<< "$publish_output"; then
        # Dry-run limitation: dependent crates cannot resolve unpublished
        # workspace dependencies on the registry. The real publish sequence
        # resolves them because earlier crates in CRATES are already indexed.
        echo "OK   $crate (dry-run limited: depends on unpublished workspace crates)"
    else
        echo "FAIL $crate"
        echo "$publish_output" | tail -5
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