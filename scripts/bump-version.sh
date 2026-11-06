#!/usr/bin/env bash
set -euo pipefail

# bump-version.sh
# Bumps version across all workspace crates consistently.
#
# Usage: ./scripts/bump-version.sh <new-version>
#   e.g., ./scripts/bump-version.sh 0.3.0
#
# Updates all Cargo.toml version fields + workspace dependency versions.
# Does NOT create a git commit.

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <new-version>"
    echo "  e.g., $0 0.3.0"
    exit 1
fi

NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Validate semver format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: '$NEW_VERSION' is not a valid semver version"
    exit 1
fi

# Read the workspace version the crate manifests inherit.
CURRENT_VERSION=$(awk '
    /^\[workspace\.package\]/ { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^version = "/ {
        match($0, /"[^"]+"/)
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
    }
' "$REPO_ROOT/Cargo.toml")

if [[ -z "$CURRENT_VERSION" ]]; then
    echo "Error: the workspace version could not be read from $REPO_ROOT/Cargo.toml"
    exit 1
fi

if [[ "$CURRENT_VERSION" == "$NEW_VERSION" ]]; then
    echo "Already at version $NEW_VERSION, nothing to do."
    exit 0
fi

echo "Bumping the workspace from $CURRENT_VERSION to $NEW_VERSION"

# Every crate inherits the workspace version, so exactly one manifest line and
# every inter-crate dependency declaration change.
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$REPO_ROOT/Cargo.toml"
rm -f "$REPO_ROOT/Cargo.toml.bak"
sed -i.bak "s|version = \"$CURRENT_VERSION\" }|version = \"$NEW_VERSION\" }|g" "$REPO_ROOT/Cargo.toml"
rm -f "$REPO_ROOT/Cargo.toml.bak"

if ! grep -q "^version = \"$NEW_VERSION\"" "$REPO_ROOT/Cargo.toml"; then
    echo "Error: the workspace version was not updated"
    exit 1
fi
if grep -q "version = \"$CURRENT_VERSION\"" "$REPO_ROOT/Cargo.toml"; then
    echo "Error: a workspace dependency still pins $CURRENT_VERSION"
    exit 1
fi

echo ""
echo "All crates bumped to $NEW_VERSION"
echo "Review changes with: git diff"
echo "Then commit: git add -A && git commit -m 'chore: bump version to $NEW_VERSION'"