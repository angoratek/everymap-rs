#!/usr/bin/env bash
set -euo pipefail

# check-publish-readiness.sh
# Validates that all publishable workspace crates have required metadata.
# Exit 0 if ready, exit 1 with details if not.
#
# Usage: ./scripts/check-publish-readiness.sh [--verbose]

VERBOSE=false
[[ "${1:-}" == "--verbose" ]] && VERBOSE=true

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
TOTAL=0
SKIPPED=0

log() { echo "  $1"; }

check_crate() {
    local crate_dir="$1"
    local crate_name
    crate_name=$(grep '^name' "$crate_dir/Cargo.toml" | head -1 | sed 's/name = "\(.*\)"/\1/' || basename "$crate_dir")

    # Skip publish = false crates
    if grep -q 'publish\s*=\s*false' "$crate_dir/Cargo.toml" 2>/dev/null; then
        if $VERBOSE; then log "SKIP $crate_name (publish = false)"; fi
        SKIPPED=$((SKIPPED + 1))
        return 0
    fi

    TOTAL=$((TOTAL + 1))
    local errors=()

    # Check required fields (accepts `field = "..."` and workspace-inherited
    # `field.workspace = true`, which must be defined in [workspace.package])
    for field in description license repository; do
        if ! grep -qE "^${field}(\.workspace)?\s*=" "$crate_dir/Cargo.toml"; then
            errors+=("missing required field: $field")
        elif grep -q "^${field}\.workspace\s*=" "$crate_dir/Cargo.toml" \
            && ! grep -qE "^${field}\s*=" "$REPO_ROOT/Cargo.toml"; then
            errors+=("$field.workspace = true but '$field' not defined in [workspace.package]")
        fi
    done

    # Check wiremock is not in [dependencies] (only in dev-dependencies)
    if [ -f "$crate_dir/Cargo.toml" ]; then
        local in_deps=false
        local section=""
        while IFS= read -r line; do
            if [[ "$line" =~ ^\[dependencies\] ]]; then section="deps"
            elif [[ "$line" =~ ^\[dev-dependencies\] ]]; then section="dev"
            elif [[ "$line" =~ ^\[ ]]; then section=""
            elif [[ "$line" =~ wiremock ]]; then
                if [[ "$section" == "deps" ]]; then in_deps=true; fi
            fi
        done < "$crate_dir/Cargo.toml"
        if $in_deps; then
            errors+=("wiremock in [dependencies] (should be [dev-dependencies])")
        fi
    fi

    # Check inter-crate deps have version specified ([dependencies] only —
    # path-only dev-dependencies are intentional and stripped at publish)
    local in_deps_section=false
    local section_name=""
    while IFS= read -r line; do
        if [[ "$line" =~ ^\[dependencies\] ]]; then in_deps_section=true; section_name="deps"
        elif [[ "$line" =~ ^\[dev-dependencies\] ]]; then in_deps_section=false; section_name="dev"
        elif [[ "$line" =~ ^\[ ]]; then in_deps_section=false; section_name=""
        elif $in_deps_section && [[ "$line" =~ ^everymap-.*path ]]; then
            if ! [[ "$line" =~ version ]]; then
                errors+=("inter-crate dependency has path without version (need path+version for crates.io)")
            fi
        fi
    done < "$crate_dir/Cargo.toml"

    # Check crates.io discovery metadata: keywords (max 5), categories,
    # readme, and documentation links
    local keyword_count
    keyword_count=$(grep -E '^keywords' "$crate_dir/Cargo.toml" | grep -o '"' | wc -l | tr -d ' ')
    keyword_count=$((keyword_count / 2))
    if [[ $keyword_count -eq 0 ]]; then
        errors+=("missing keywords (crates.io search discoverability)")
    elif [[ $keyword_count -gt 5 ]]; then
        errors+=("too many keywords: ${keyword_count} (crates.io allows max 5)")
    fi
    if ! grep -qE '^categories' "$crate_dir/Cargo.toml"; then
        errors+=("missing categories (crates.io browse discoverability)")
    fi
    if ! grep -qE '^readme(\.workspace)?\s*=' "$crate_dir/Cargo.toml"; then
        errors+=("missing readme field (crates.io page renders no README)")
    fi
    if ! grep -qE '^documentation\s*=' "$crate_dir/Cargo.toml"; then
        errors+=("missing documentation field (no docs.rs link on crates.io)")
    fi

    # Check LICENSE file at repo root
    if [[ ! -f "$REPO_ROOT/LICENSE" ]]; then
        errors+=("no LICENSE file in repo root")
    fi

    if [[ ${#errors[@]} -eq 0 ]]; then
        log "OK   $crate_name"
    else
        log "FAIL $crate_name"
        FAILED=$((FAILED + 1))
        for err in "${errors[@]}"; do
            log "     - $err"
        done
    fi
}

echo "Checking publish readiness for all workspace crates..."
for crate_dir in "$REPO_ROOT"/everymap-*/; do
    check_crate "$crate_dir"
done

echo ""
echo "Result: $((TOTAL - FAILED))/$TOTAL crates ready, $SKIPPED skipped"
if [[ $FAILED -gt 0 ]]; then
    exit 1
fi