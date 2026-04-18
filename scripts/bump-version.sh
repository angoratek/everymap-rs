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

# Get current version from any crate
CURRENT_VERSION=$(grep '^version' "$REPO_ROOT/everymap-core/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

if [[ "$CURRENT_VERSION" == "$NEW_VERSION" ]]; then
    echo "Already at version $NEW_VERSION, nothing to do."
    exit 0
fi

echo "Bumping all crates from $CURRENT_VERSION to $NEW_VERSION"

# Update version in each crate's Cargo.toml
for crate_dir in "$REPO_ROOT"/everymap-*/; do
    crate_name=$(basename "$crate_dir")
    echo "  $crate_name"
    sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$crate_dir/Cargo.toml"
    rm -f "$crate_dir/Cargo.toml.bak"
done

# Update workspace dependency versions in root Cargo.toml
sed -i.bak "s|version = \"$CURRENT_VERSION\" }|version = \"$NEW_VERSION\" }|g" "$REPO_ROOT/Cargo.toml"
rm -f "$REPO_ROOT/Cargo.toml.bak"

echo ""
echo "All crates bumped to $NEW_VERSION"
echo "Review changes with: git diff"
echo "Then commit: git add -A && git commit -m 'chore: bump version to $NEW_VERSION'"