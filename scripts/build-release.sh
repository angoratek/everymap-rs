#!/usr/bin/env bash
set -euo pipefail

# build-release.sh
# Cross-compiles everymap-cli binary for multiple platforms.
#
# Usage: ./scripts/build-release.sh [--target TARGET] [--version VERSION]
#   --target     Build only for specified target (e.g., x86_64-unknown-linux-gnu)
#   --version    Override version string (defaults to Cargo.toml version)

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="$REPO_ROOT/target/release-artifacts"
BIN_NAME="everymap"

DEFAULT_TARGETS=(
    x86_64-unknown-linux-gnu
    aarch64-unknown-linux-gnu
    x86_64-apple-darwin
    aarch64-apple-darwin
    x86_64-pc-windows-msvc
)

TARGETS=("${DEFAULT_TARGETS[@]}")
VERSION=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target) TARGETS=("$2"); shift 2;;
        --version) VERSION="$2"; shift 2;;
        --help) echo "Usage: $0 [--target TARGET] [--version VERSION]"; exit 0;;
        *) echo "Unknown arg: $1"; exit 1;;
    esac
done

if [[ -z "$VERSION" ]]; then
    VERSION=$(grep '^version' "$REPO_ROOT/everymap-cli/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

mkdir -p "$OUTPUT_DIR"

for target in "${TARGETS[@]}"; do
    echo "=== Building for $target ==="

    # Install target if not present
    rustup target list --installed | grep -q "$target" || rustup target add "$target"

    # Build
    cargo build --release --target "$target" --manifest-path "$REPO_ROOT/everymap-cli/Cargo.toml"

    # Determine binary name (windows gets .exe)
    local_bin="$BIN_NAME"
    [[ "$target" == *windows* ]] && local_bin="${BIN_NAME}.exe"

    # Copy to output with versioned name
    output_name="everymap-${VERSION}-${target}"
    [[ "$target" == *windows* ]] && output_name="${output_name}.exe"

    cp "$REPO_ROOT/target/$target/release/$local_bin" "$OUTPUT_DIR/$output_name"

    # Generate sha256
    shasum -a 256 "$OUTPUT_DIR/$output_name" > "$OUTPUT_DIR/$output_name.sha256"

    echo "Built: $OUTPUT_DIR/$output_name"
done

echo ""
echo "=== All builds complete ==="
ls -la "$OUTPUT_DIR/"