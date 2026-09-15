#!/usr/bin/env bash
set -euo pipefail

# Generates per-crate READMEs for the 7 published EveryMap crates by
# concatenating docs/readme/intro.md with the per-crate body fragment.
#
# Usage:
#   scripts/generate-readmes.sh           regenerate <crate>/README.md
#   scripts/generate-readmes.sh --check   verify generated files match the fragments (exit 1 on drift)

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fragments_dir="$repo_root/docs/readme"

crates=(
  everymap-core
  everymap-providers-here
  everymap-providers-google
  everymap-providers-tomtom
  everymap-providers-mapbox
  everymap-providers-radar
  everymap-cli
)

check_mode=false
if [[ "${1:-}" == "--check" ]]; then
  check_mode=true
elif [[ -n "${1:-}" ]]; then
  echo "usage: scripts/generate-readmes.sh [--check]" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

failed=0
for crate in "${crates[@]}"; do
  body="$fragments_dir/$crate.md"
  if [[ ! -f "$body" ]]; then
    echo "error: missing fragment: $body" >&2
    failed=1
    continue
  fi

  generated="$tmp_dir/$crate/README.md"
  mkdir -p "$(dirname "$generated")"
  { cat "$fragments_dir/intro.md"; echo; cat "$body"; } > "$generated"

  target="$repo_root/$crate/README.md"
  if $check_mode; then
    if [[ ! -f "$target" ]]; then
      echo "error: missing generated file: $target (run scripts/generate-readmes.sh)" >&2
      failed=1
    elif ! diff -u "$target" "$generated"; then
      echo "error: $target is out of date with docs/readme fragments" >&2
      failed=1
    fi
  else
    cp "$generated" "$target"
    echo "generated $target"
  fi
done

if $check_mode; then
  if (( failed )); then
    echo "readme check FAILED — run scripts/generate-readmes.sh to regenerate" >&2
  else
    echo "readme check passed"
  fi
fi
exit "$failed"