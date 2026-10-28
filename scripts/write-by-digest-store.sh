#!/usr/bin/env bash
# Writes the registry's by-digest store: every bundle listed in
# <specs-dir>/index.json copied to <specs-dir>/by-digest/sha256-<hex>.json, its
# content address (docs/design/consumer-side-generation.md, milestone 2).
#
#   scripts/write-by-digest-store.sh website/specs
#
# Run after the bundles are built (`npm run build-bundles`). Each copy is
# re-hashed, so a name can never promise other content. Used by deploy.yml,
# which uploads the store append-only, and by rust.yml's trust-tasks-cli job,
# which tests the CLI against the whole registry.

set -euo pipefail

specs="${1:?usage: $0 <specs-dir>}"
index="$specs/index.json"
[[ -f "$index" ]] || { echo "no $index — build the bundles first" >&2; exit 1; }

if sha256sum --version 2>/dev/null | grep -q GNU; then
  sha=(sha256sum)        # GNU coreutils
else
  sha=(shasum -a 256)    # macOS
fi

mkdir -p "$specs/by-digest"
jq -r '.specs[] | "\(.digest | ltrimstr("sha256:")) \(.path)"' "$index" |
  while read -r hex path; do
    out="$specs/by-digest/sha256-$hex.json"
    cp "$specs/$path" "$out"
    echo "$hex  $out" | "${sha[@]}" --check --quiet --strict
  done
echo "by-digest store: $(find "$specs/by-digest" -name 'sha256-*.json' | wc -l | tr -d ' ') bundles in $specs/by-digest"
