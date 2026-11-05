#!/usr/bin/env bash
set -euo pipefail

# check-provider-transport-isolation.sh
#
# A provider crate reaches the network only through `everymap_core`'s
# transport seam (`ProviderClient`), never through `reqwest` or `tokio`
# directly. That is what lets one provider crate build and run natively, in a
# browser and on a mobile host without per-target code.
#
# Test modules may still use `#[tokio::test]` natively.
#
# Usage: scripts/check-provider-transport-isolation.sh

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if matches=$(grep -rn --include='*.rs' -E '\b(reqwest|tokio)::' \
        "${repo_root}"/everymap-providers-*/src 2>/dev/null \
    | grep -v '#\[tokio::test\]'); then
    printf '%s\n' "$matches" >&2
    printf '%s\n' \
        'provider-transport-isolation: a provider crate names reqwest or tokio directly.' \
        'Route the request through everymap_core::client::ProviderClient instead so the crate builds and runs on every target.' >&2
    exit 1
fi

printf '%s\n' 'provider-transport-isolation: every provider goes through the transport seam.'
