#!/usr/bin/env bash
#
# Applies the next release of one npm package to the working tree.
#
#   scripts/release-bump-ts.sh [@openvtc/trust-tasks|@openvtc/trust-tasks-proof|...]
#
# Called by scripts/release-pr.sh, once per package, while it assembles the ONE
# Release PR. This script only edits files; the orchestrator commits and opens
# the PR. release-plz cannot do this part — it is Rust-only and will not touch
# a non-Rust manifest — so the npm bump is computed here, deliberately the same
# way release-plz computes the crates' so every part of a release reads alike:
#
#   * the previous release is a tag  — `trust-tasks-ts-v<version>`, written by
#     the `publish-npm` job, mirroring release-plz's `<crate>-v<version>`;
#   * the bump comes from conventional commits since that tag, over the paths
#     that can change what the package publishes;
#   * the changelog section comes from git-cliff with the same `cliff.toml`
#     release-plz uses;
#   * merging the Release PR is the release — `publish-npm` sees a version that
#     is not on npm and publishes it.
#
# It is idempotent: the target version is computed from the last TAG, never
# from the branch, so re-running on a fresh checkout recomputes the same answer.
# It exits 0 without touching anything when there is nothing to release. When
# it does bump, it appends one table row to $RELEASE_SUMMARY.
#
# Run from the repository root, on a full-history checkout of `main`, with
# `git-cliff` on PATH.

set -euo pipefail

# Which npm package to release. The core defaults; the sibling packages
# (@openvtc/trust-tasks-proof, @openvtc/trust-tasks-tsp) pass their own name.
# Each has its own directory, tag prefix and watch set — the one place they
# differ — mirroring scripts/release-bump-dart.sh.
PKG="${1:-@openvtc/trust-tasks}"
case "$PKG" in
  @openvtc/trust-tasks)
    PKG_DIR="trust-tasks-ts"
    TAG_PREFIX="trust-tasks-ts-v"
    # specs/** and the generator because the bindings under trust-tasks-ts/src
    # are generated from them.
    WATCH=("specs" "$PKG_DIR" "scripts/build-ts-bindings.mjs")
    ;;
  @openvtc/trust-tasks-proof)
    PKG_DIR="trust-tasks-ts-proof"
    TAG_PREFIX="trust-tasks-ts-proof-v"
    WATCH=("$PKG_DIR")
    ;;
  @openvtc/trust-tasks-tsp)
    PKG_DIR="trust-tasks-ts-tsp"
    TAG_PREFIX="trust-tasks-ts-tsp-v"
    WATCH=("$PKG_DIR")
    ;;
  @openvtc/trust-tasks-capability-client)
    PKG_DIR="trust-tasks-ts-capability-client"
    TAG_PREFIX="trust-tasks-ts-capability-client-v"
    WATCH=("$PKG_DIR")
    ;;
  *)
    echo "::error::unknown TS package '$PKG'"
    exit 1
    ;;
esac
MANIFEST="$PKG_DIR/package.json"
CHANGELOG="$PKG_DIR/CHANGELOG.md"

name=$(node -p "require('./$MANIFEST').name")
current=$(node -p "require('./$MANIFEST').version")

# ── Where the last release was ───────────────────────────────────────────────
tag=$(git tag -l "${TAG_PREFIX}*" --sort=-version:refname | head -1)
if [ -z "$tag" ]; then
  echo "::error::No ${TAG_PREFIX}* tag exists, so there is no anchor to measure this release from. Seed it once at the currently-published version — see RELEASING.md, 'One-time migration'."
  exit 1
fi
last="${tag#"$TAG_PREFIX"}"

if [ "$current" != "$last" ]; then
  # main already carries a version newer than the last tag: a release is merged
  # but not yet on npm (publish-npm failed, or is still running). Proposing
  # another bump on top would release two versions for one set of changes.
  echo "::warning::$MANIFEST is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the publish-npm job rather than bumping it again."
  exit 0
fi

# ── Is there anything to release? ────────────────────────────────────────────
if git diff --quiet "$tag" HEAD -- "${WATCH[@]}"; then
  echo "::notice::No changes to $name since $tag — nothing to release."
  exit 0
fi

# ── What size of bump? ───────────────────────────────────────────────────────
# Conventional commits, same signal release-plz reads for the crates: a `!`
# after the type/scope or a `BREAKING CHANGE:` trailer means the compatibility
# field moves; anything else is a patch. Note that unlike the crate side there
# is no cargo-semver-checks equivalent here to catch an unannounced break, so
# this is only as accurate as the commit subjects. See RELEASING.md.
range="$tag..HEAD"
# The log is captured first, NOT piped into `grep -q`: grep exits on its first
# match, git log then dies of SIGPIPE, and under `pipefail` the pipeline reports
# failure — so a breaking change was silently scored as a patch whenever the log
# was long enough to still be writing. It happened to #610 (`feat(persona)!`).
log=$(git log --format='%s%n%b' "$range" -- "${WATCH[@]}")
if grep -qE '^[a-z]+(\([^)]*\))?!:|^BREAKING[ -]CHANGE' <<<"$log"; then
  level=breaking
else
  level=patch
fi

next=$(LEVEL="$level" CURRENT="$last" node -e '
  const [maj, min, pat] = process.env.CURRENT.split(".").map(Number);
  const breaking = process.env.LEVEL === "breaking";
  // Cargo/npm 0.x rule, the same one release-plz applies to these crates: below
  // 1.0 the leading non-zero component is the compatibility boundary, so a
  // break moves the MINOR field and everything else moves the patch.
  // 0.0.x is a special case: cargo/npm treat every 0.0.x as incompatible with
  // every other, so the patch field already IS the compatibility boundary and
  // moving the minor would over-bump.
  if (!breaking || (maj === 0 && min === 0)) { console.log(`${maj}.${min}.${pat + 1}`); }
  else if (maj === 0) { console.log(`0.${min + 1}.0`); }
  else { console.log(`${maj + 1}.0.0`); }
')

echo "::notice::$name $last -> $next ($level)"

# ── Apply the bump ───────────────────────────────────────────────────────────
# In the working tree only. scripts/release-pr.sh commits every package's bump
# together, as the one Release PR.

node -e '
  const fs = require("fs");
  const p = process.argv[1], v = process.argv[2];
  const raw = fs.readFileSync(p, "utf8");
  // Rewrite only the top-level "version" field, in place, so npm formatting
  // and key order survive. JSON.parse/stringify would reorder and reindent the
  // whole manifest and bury the one-line change CI is meant to review.
  const out = raw.replace(/^(\s*"version"\s*:\s*")[^"]*(")/m, `$1${v}$2`);
  if (out === raw) { console.error(`could not rewrite version in ${p}`); process.exit(1); }
  fs.writeFileSync(p, out);
' "$MANIFEST" "$next"

# Keep the lockfile's self-reference honest; `npm ci` in the publish job reads
# it and a stale version there is a mismatch waiting to happen.
if [ -f "$PKG_DIR/package-lock.json" ]; then
  (cd "$PKG_DIR" && npm install --package-lock-only --ignore-scripts --no-audit --no-fund) \
    || echo "::warning::could not refresh $PKG_DIR/package-lock.json"
fi

# Changelog generation is best-effort ON PURPOSE. The version bump is the
# load-bearing part of this PR — it is what reaches npm — and a git-cliff
# hiccup must not be able to block a release. A missing section is visible in
# the PR diff and can be written by hand.
cliff_paths=()
for w in "${WATCH[@]}"; do
  if [ -d "$w" ]; then cliff_paths+=(--include-path "$w/**"); else cliff_paths+=(--include-path "$w"); fi
done
if section=$(git-cliff --config cliff.toml \
  --strip header \
  --tag "v$next" \
  --unreleased \
  "${cliff_paths[@]}" \
  "$range" 2>/dev/null) && [ -n "$section" ]; then
  SECTION="$section" node -e '
    const fs = require("fs");
    const p = process.argv[1];
    const section = process.env.SECTION.trim() + "\n\n";
    const raw = fs.readFileSync(p, "utf8");
    // Insert above the newest existing release heading, keeping the file
    // preamble (which explains the versioning rule) at the top.
    const i = raw.indexOf("\n## ");
    fs.writeFileSync(p, i === -1 ? raw.trimEnd() + "\n\n" + section : raw.slice(0, i + 1) + section + raw.slice(i + 1));
  ' "$CHANGELOG"
else
  echo "::warning::git-cliff produced no changelog section for $range; the Release PR carries the version bump only."
fi


# One row of the Release PR's table; scripts/release-pr.sh owns the rest.
printf '| `%s` | npm | `%s` → `%s` | %s |\n' "$name" "$last" "$next" "$level" >>"${RELEASE_SUMMARY:-/dev/stdout}"
