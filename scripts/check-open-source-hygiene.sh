#!/usr/bin/env bash
set -euo pipefail

# check-open-source-hygiene.sh
#
# Every tracked file in this repository is published (crates.io, GitHub, the
# generated documentation), so it must not carry a credential, a personal
# filesystem path or a private note. This gate scans the tracked tree — the
# planning documents included — for the patterns that have leaked before.
#
# False positives are fixed by rewording the file, not by weakening the gate.
#
# Usage: scripts/check-open-source-hygiene.sh

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Patterns that must never appear in a published file.
forbidden_patterns=(
    '/Users/[A-Za-z0-9._-]+'
    '/home/[A-Za-z0-9._-]+'
    'BEGIN [A-Z ]*PRIVATE KEY'
    'AKIA[0-9A-Z]{16}'
    'ghp_[A-Za-z0-9]{36}'
    'sk-[A-Za-z0-9]{20,}'
    # A recorded provider payload may keep the shape of a response field but
    # never a live opaque token: a session id is credential-adjacent account
    # material, and the same value appears in the image URLs beside it.
    '"sessionId"[[:space:]]*:[[:space:]]*"[A-Za-z0-9_-]{40,}'
    'session=[A-Za-z0-9_.~-]{40,}'
    'signature=[A-Za-z0-9%._~-]{40,}'
    # This library keeps its own identity: a consumer SDK's name, namespace or
    # repository never appears in its sources, documents or generated pages.
    'Rustic Maps'
    'rustic-maps-sdk'
    'rustic_maps'
)

violations=0
for pattern in "${forbidden_patterns[@]}"; do
    if matches=$(git grep -InE "$pattern" -- . ':!Cargo.lock' ':!scripts/check-open-source-hygiene.sh' 2>/dev/null); then
        printf '%s\n' "$matches" >&2
        printf 'open-source-hygiene: the pattern /%s/ appears above.\n' "$pattern" >&2
        violations=1
    fi
done

# The tracked tree above skips binary files. A recorded payload or an
# artifact can embed the same values, so scan every tracked file as text.
for pattern in "${forbidden_patterns[@]}"; do
    if matches=$(git grep -aE "$pattern" -- . ':!Cargo.lock' ':!scripts/check-open-source-hygiene.sh' 2>/dev/null); then
        printf '%s\n' "$matches" >&2
        printf 'open-source-hygiene: the pattern /%s/ appears in a binary file above.\n' "$pattern" >&2
        violations=1
    fi
done

# Commit metadata is published too, and a message can carry a path or a key.
for pattern in "${forbidden_patterns[@]}"; do
    if matches=$(git log --format='%H %B' | grep -E "$pattern"); then
        printf '%s\n' "$matches" >&2
        printf 'open-source-hygiene: the pattern /%s/ appears in commit metadata above.\n' "$pattern" >&2
        violations=1
    fi
done

if [ "$violations" -ne 0 ]; then
    printf '%s\n' \
        'open-source-hygiene: a published file carries a path, a key shape or a secret marker.' \
        'Reword the file; do not weaken the gate.' >&2
    exit 1
fi

printf '%s\n' 'open-source-hygiene: no credential, personal path or private marker in the tracked tree.'
