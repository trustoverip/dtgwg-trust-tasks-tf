#!/usr/bin/env bash
#
# Applies the next release of one Dart package to the working tree.
#
#   scripts/release-bump-dart.sh [trust_tasks|trust_tasks_proof|trust_tasks_https|...]
#
# Called by scripts/release-pr.sh, once per package, while it assembles the ONE
# Release PR; this script only edits files. Defaults to `trust_tasks`. Each
# package has its own tag prefix and watch set and is published to pub.dev on
# its own tag; the package table below is the single place they differ. The
# bump is computed the same way as scripts/release-bump-ts.sh and -go.sh, so
# every part of a release reads identically — see RELEASING.md.
#
# ── What is different about Dart ─────────────────────────────────────────────
#
# pub.dev only accepts an automated publish when the workflow run was triggered
# by pushing a git TAG, which it verifies in the GitHub OIDC token's claims. So
# unlike the npm side — which publishes as soon as the Release PR merges — the
# merge makes `tag-dart` write `trust-tasks-dart-v<version>`, and that tag
# triggers `publish-dart.yml`, which publishes.
#
# The consequence worth knowing: a tag pushed by the default GITHUB_TOKEN does
# not trigger a workflow, so without RELEASE_PLZ_TOKEN set the tag lands and the
# publish does not. `tag-dart` warns loudly in that case and the fix is to
# re-push the tag from a workstation.
#
# For `trust_tasks`, two versions must move together: `version:` in pubspec.yaml
# (what pub.dev publishes) and `packageVersion` in lib/src/runtime/version.dart
# (what a consumer can log). `npm run check-bindings` fails if they disagree.
# The other packages carry no version constant.
#
# It is idempotent: the target version is computed from the last TAG, never from
# the branch, so re-running on a fresh checkout recomputes the same answer. It
# exits 0 without touching anything when there is nothing to release; when it
# does bump, it appends one table row to $RELEASE_SUMMARY.
#
# Run from the repository root, on a full-history checkout of `main`, with
# `git-cliff` on PATH.

set -euo pipefail

PKG="${1:-trust_tasks}"

# WATCH is everything that can change what the package publishes.
case "$PKG" in
  trust_tasks)
    PKG_DIR="trust-tasks-dart"
    VERSION_FILE="$PKG_DIR/lib/src/runtime/version.dart"
    # `specs/**` because the libraries under `trust-tasks-dart/lib/specs` are
    # generated from it, and `scripts/build-dart-bindings.mjs` because it is the
    # generator.
    WATCH=("specs" "$PKG_DIR" "scripts/build-dart-bindings.mjs")
    ;;
  trust_tasks_proof)
    PKG_DIR="trust-tasks-dart-proof"
    VERSION_FILE=""
    # Only its own tree. A core change that alters this package's behaviour
    # reaches consumers through the core's release, not this one — the same
    # separation as between trust-tasks-rs and trust-tasks-proof.
    WATCH=("$PKG_DIR")
    ;;
  trust_tasks_https)
    PKG_DIR="trust-tasks-dart-https"
    VERSION_FILE=""
    # Only its own tree, for the same reason as trust_tasks_proof.
    WATCH=("$PKG_DIR")
    ;;
  trust_tasks_didcomm)
    PKG_DIR="trust-tasks-dart-didcomm"
    VERSION_FILE=""
    WATCH=("$PKG_DIR")
    ;;
  trust_tasks_capability_client)
    PKG_DIR="trust-tasks-dart-capability-client"
    VERSION_FILE=""
    WATCH=("$PKG_DIR")
    ;;
  *)
    echo "::error::unknown Dart package '$PKG'"
    exit 1
    ;;
esac

PUBSPEC="$PKG_DIR/pubspec.yaml"
CHANGELOG="$PKG_DIR/CHANGELOG.md"
TAG_PREFIX="$PKG_DIR-v"

current=$(sed -n 's/^version: *//p' "$PUBSPEC" | head -1 | tr -d '\r')
if [ -z "$current" ]; then
  echo "::error::Could not read \`version\` from $PUBSPEC."
  exit 1
fi

# ── Where the last release was ───────────────────────────────────────────────
tag=$(git tag -l "${TAG_PREFIX}*" --sort=-version:refname | head -1)
if [ -z "$tag" ]; then
  echo "::error::No ${TAG_PREFIX}* tag exists. tag-dart writes it once the package has a version with no matching tag — re-run that job rather than tagging by hand, because the tag is what pub.dev publishes from. See RELEASING.md."
  exit 1
fi
last="${tag#"$TAG_PREFIX"}"

if [ "$current" != "$last" ]; then
  # main already carries a version newer than the last tag: a release is merged
  # but not yet tagged. Proposing another bump on top would release two versions
  # for one set of changes.
  echo "::warning::$PUBSPEC is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the tag-dart job rather than bumping it again."
  exit 0
fi

# ── Is there anything to release? ────────────────────────────────────────────
if git diff --quiet "$tag" HEAD -- "${WATCH[@]}"; then
  echo "::notice::No changes to $PKG since $tag — nothing to release."
  exit 0
fi

# ── What size of bump? ───────────────────────────────────────────────────────
# Conventional commits, the same signal release-plz reads for the crates. As on
# the npm and Go sides there is no cargo-semver-checks equivalent to catch an
# unannounced break, so this is only as accurate as the commit subjects.
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
  // Below 1.0 the leading non-zero component is the compatibility boundary, the
  // same rule the other three sides apply. 0.0.x is a special case: every 0.0.x
  // is incompatible with every other, so the patch field already IS the
  // boundary and moving the minor would over-bump.
  if (!breaking || (maj === 0 && min === 0)) { console.log(`${maj}.${min}.${pat + 1}`); }
  else if (maj === 0) { console.log(`0.${min + 1}.0`); }
  else { console.log(`${maj + 1}.0.0`); }
')

echo "::notice::$PKG $last -> $next ($level)"

# ── Apply the bump ───────────────────────────────────────────────────────────
# In the working tree only. scripts/release-pr.sh commits every package's bump
# together, as the one Release PR.

NEXT="$next" node -e '
  const fs = require("fs");
  const [pubspec, versionFile] = process.argv.slice(1);
  const v = process.env.NEXT;

  // Rewrite only the top-level `version:` line, so comments and key order
  // survive and the one-line change CI reviews stays one line.
  const raw = fs.readFileSync(pubspec, "utf8");
  const out = raw.replace(/^version: .*$/m, `version: ${v}`);
  if (out === raw) { console.error(`could not rewrite version in ${pubspec}`); process.exit(1); }
  fs.writeFileSync(pubspec, out);

  // And the constant that mirrors it, leaving its doc comment untouched —
  // where the package has one. (An `if` block, not an early `return`: `node -e`
  // runs this as a script, where a top-level return is a syntax error.)
  //
  // \x27 is a single quote: the JS below is already inside single quotes in the
  // shell, so a literal one would end the argument. The first attempt at this
  // regex used `[^.]*` for the version, which cannot match `0.1.0` at all — it
  // silently rewrote nothing, which at release time would have shipped a
  // pubspec and a constant that disagreed.
  if (versionFile) {
    const src = fs.readFileSync(versionFile, "utf8");
    const next = src.replace(
      /^(const String packageVersion = \x27)[^\x27]*(\x27;)$/m,
      `$1${v}$2`,
    );
    if (next === src) { console.error(`could not rewrite packageVersion in ${versionFile}`); process.exit(1); }
    fs.writeFileSync(versionFile, next);
  }
' "$PUBSPEC" "$VERSION_FILE"

# Changelog generation is best-effort ON PURPOSE. The version bump is the
# load-bearing part of this PR — it is what decides the tag — and a git-cliff
# hiccup must not be able to block a release. A missing section is visible in
# the PR diff and can be written by hand.
# The same paths as WATCH, as git-cliff globs.
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
    const i = raw.indexOf("\n## ");
    fs.writeFileSync(p, i === -1 ? raw.trimEnd() + "\n\n" + section : raw.slice(0, i + 1) + section + raw.slice(i + 1));
  ' "$CHANGELOG"
else
  echo "::warning::git-cliff produced no changelog section for $range; the Release PR carries the version bump only."
fi


# One row of the Release PR's table; scripts/release-pr.sh owns the rest.
printf '| `%s` | pub.dev | `%s` → `%s` | %s |\n' "$PKG" "$last" "$next" "$level" >>"${RELEASE_SUMMARY:-/dev/stdout}"
