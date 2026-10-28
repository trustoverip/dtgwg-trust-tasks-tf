#!/usr/bin/env bash
#
# Fails a pull request that breaks the public API of a published crate without
# declaring it.
#
#   PR_TITLE='feat(rs)!: …' scripts/check-rs-api-compat.sh
#
# Environment:
#   PR_TITLE     the pull request title (REQUIRED; passed through the
#                environment, never spliced into a workflow `run:`)
#   BASE_REF     the PR's base commit, for the "already breaking" history check
#                and for deciding which binding crates changed (default: HEAD)
#
# Run from the repository root with the `<crate>-v*` tags present
# (`fetch-depth: 0` in CI).
#
# ── Why here, and not in the Release PR ──────────────────────────────────────
#
# Until this script, release-plz ran cargo-semver-checks itself while building
# the Release PR (`semver_check = true`), after every merge to main. Over the
# whole generated tree that is rustdoc JSON for ~90 MB of Rust, twice, peaking
# at ~12.6 GB: the job took 8-13 minutes and was repeatedly killed for memory
# on a 16 GB runner. And it found a break only after the PR that made it had
# merged. The npm, Go and Dart packages are checked on the PR instead
# (scripts/check-go-api-compat.sh and friends); this puts the crates with them,
# and release-plz.toml now has `semver_check = false`.
#
# ── What is compared, and why it is enough ──────────────────────────────────
#
# Each crate against its own last release, `--baseline-rev <crate>-v<x.y.z>`.
#
#   trust-tasks-rs   the hand-written runtime plus ONE generated family (`acl`)
#                    and `validate`. The other families are the same generator's
#                    output over other schemas, so a codegen change that breaks
#                    one breaks `acl` too; a schema change that would break one
#                    is refused by `published schemas change only additively`;
#                    and a withdrawn version is removed in the batch PR, whose
#                    title already declares the break. ~200 MB, seconds, against
#                    ~12.6 GB for every family.
#   trust-tasks      the facade, `--only-explicit-features`: its dependency on
#                    trust-tasks-rs is `default-features = false`, so this stays
#                    as small; what it adds is re-exports and its feature list.
#   trust-tasks-ceremony, trust-tasks-build
#                    no trust-tasks-rs dependency; cheap, always checked.
#                    (trust-tasks-cli is a binary: it has no library API.)
#   the bindings     (https, proof, didcomm, didcomm-v1, tsp, capability-client)
#                    depend on trust-tasks-rs with every family, so documenting
#                    one compiles the whole generated tree. Checked only when the
#                    PR changes that crate's own directory — cargo-semver-checks
#                    compares a crate's own items and cannot see a break that
#                    arrives through a dependency's types anyway; that is what
#                    the `core` version group in release-plz.toml is for.
#
# A crate with no release tag yet (a new crate before its manual first publish)
# is skipped.
#
# ── The verdict, per crate ───────────────────────────────────────────────────
#
#   no major-level change                          → pass
#   major-level change, title `<type>(<scope>)!:` with a scope that releases
#     this crate (below)                           → pass, listing the breaks
#   major-level change, and a commit already on the base branch since the tag
#     declared one                                 → pass: the release is breaking anyway
#   otherwise                                      → FAIL
#
# The core crates release as one version group, so a `!` on any of their scopes
# declares a break for all of them. trust-tasks-ceremony (`ceremony`) and
# trust-tasks-build (`build`) version on their own.
#
# On the Release PR itself the versions have moved, and cargo-semver-checks is
# left to judge the change against the new version, as release-plz would have.
#
# What this cannot see: a behavioural break with no API change (a spec that
# starts requiring a proof). Declare those yourself.

set -euo pipefail

BASE_REF="${BASE_REF:-HEAD}"
CORE_SCOPES="rs codegen https didcomm didcomm-v1 tsp proof capability-client"

if [ -z "${PR_TITLE+x}" ]; then
  echo "::error::PR_TITLE is not set."
  exit 2
fi

# crate | cargo-semver-checks feature flags | always (or only when its dir changed)
CRATES=(
  "trust-tasks-rs|--only-explicit-features --features acl,validate|always"
  "trust-tasks|--only-explicit-features|always"
  "trust-tasks-ceremony||always"
  "trust-tasks-build||always"
  "trust-tasks-https||changed"
  "trust-tasks-proof||changed"
  "trust-tasks-didcomm||changed"
  "trust-tasks-didcomm-v1||changed"
  "trust-tasks-tsp||changed"
  "trust-tasks-capability-client||changed"
)

scopes_for() {
  case $1 in
    trust-tasks-ceremony) echo ceremony ;;
    trust-tasks-build) echo build ;;
    *) echo "$CORE_SCOPES" ;;
  esac
}

# The scopes of the title's `!`, if it has one (commit-lint already refuses a
# `!` whose scopes are not all packages).
title_scopes=$(printf '%s' "$PR_TITLE" | sed -nE 's/^[a-z]+\(([^)]*)\)!:.*/\1/p' | tr ',' ' ')

declares() { # declares "<scopes>" "<scopes allowed>"
  local s
  for s in $1; do
    [[ " $2 " == *" $s "* ]] && return 0
  done
  return 1
}

# A `!` with a releasing scope, or a BREAKING CHANGE trailer, merged since the
# tag on any of the crate's paths: the next release is breaking already.
already_breaking() { # already_breaking <tag> <scopes allowed> <paths…>
  local tag=$1 allowed=$2 log line
  shift 2
  log=$(git log --format='%s%n%b' "$tag..$BASE_REF" -- "$@")
  grep -qE '^BREAKING[ -]CHANGE' <<<"$log" && return 0
  while IFS= read -r line; do
    [[ $line =~ ^[a-z]+\(([^\)]*)\)!: ]] || continue
    declares "${BASH_REMATCH[1]//,/ }" "$allowed" && return 0
  done <<<"$log"
  return 1
}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

failed=0
for entry in "${CRATES[@]}"; do
  IFS='|' read -r crate flags when <<<"$entry"

  tag=$(git tag -l "${crate}-v*" | grep -E "^${crate}-v[0-9]+\.[0-9]+\.[0-9]+$" | sort -V | tail -1 || true)
  if [ -z "$tag" ]; then
    echo "::notice::$crate has no release tag yet; nothing to compare with."
    continue
  fi
  if [ "$when" = changed ] && git diff --quiet "$BASE_REF" HEAD -- "$crate/"; then
    echo "$crate: unchanged by this PR; skipped."
    continue
  fi

  released=${tag#"${crate}-v"}
  current=$(cargo metadata --no-deps --format-version 1 \
    | jq -r --arg c "$crate" '.packages[] | select(.name == $c) | .version')
  # An ordinary PR leaves the version alone, so tell cargo-semver-checks this
  # is a minor release: only a major-level change fails. A Release PR has moved
  # the version; then cargo-semver-checks judges against the version it names.
  release_type=()
  [ "$current" = "$released" ] && release_type=(--release-type minor)

  echo "::group::$crate vs $tag"
  # shellcheck disable=SC2086 # $flags is a deliberate word list
  if cargo semver-checks check-release -p "$crate" --baseline-rev "$tag" \
      $flags "${release_type[@]}" 2>&1 | tee "$work/$crate.txt"; then
    echo "::endgroup::"
    continue
  fi
  echo "::endgroup::"

  allowed=$(scopes_for "$crate")
  if declares "$title_scopes" "$allowed"; then
    echo "::notice::$crate: incompatible change(s) since $tag, declared by the PR title."
    continue
  fi
  paths=("$crate/")
  [ "$crate" = trust-tasks-rs ] && paths+=(specs trust-tasks-codegen trust-tasks-build)
  if already_breaking "$tag" "$allowed" "${paths[@]}"; then
    echo "::notice::$crate: incompatible change(s); a change merged since $tag already makes its next release breaking."
    continue
  fi

  grep -E '^--- failure ' "$work/$crate.txt" | sed "s/^/::error::$crate: undeclared API break: /" \
    || echo "::error::$crate: cargo-semver-checks failed without naming a break; see its log above."
  failed=1
done

if [ "$failed" = 1 ]; then
  cat <<EOF

This PR makes incompatible changes to the public API of the crate(s) above
since their last release, and its title does not declare a break.

Either make the change compatible (add, don't remove or rename; a new spec
version is a new module), or retitle the PR with \`!\` scoped to the package —
e.g. \`feat(rs)!: …\` — so the release is sized as breaking. See RELEASING.md.
EOF
  exit 1
fi
echo "::notice::No undeclared API break in the published crates."
