#!/usr/bin/env bash
#
# Applies the next release of one Go module to the working tree.
#
#   scripts/release-bump-go.sh [trust-tasks-go|trust-tasks-go/proof|trust-tasks-go/didcomm]
#
# Called by scripts/release-pr.sh, once per module, while it assembles the ONE
# Release PR; this script only edits files. The bump is computed the same way
# as scripts/release-bump-ts.sh and -dart.sh, so every part of a release reads
# identically — see RELEASING.md.
#
# ── What is different about Go ───────────────────────────────────────────────
#
# A Go module has no manifest to bump. The version of a Go module IS its git
# tag: `go get <module>@v1.2.3` resolves through proxy.golang.org against a tag,
# and for a module in a subdirectory that tag MUST be `<subdir>/v1.2.3` — so
# `trust-tasks-go/v1.2.3` here, with a slash, unlike the `trust-tasks-ts-v1.2.3`
# and `<crate>-v1.2.3` tags elsewhere in this repo. That spelling is the Go
# tooling's requirement, not a choice.
#
# Two consequences:
#
#   * The release tag doubles as the anchor this script measures from. The npm
#     side needs a separate `trust-tasks-ts-v*` tag written by its publish job;
#     here the tag `go get` resolves is the same tag, so there is one of them.
#   * There is nothing in the tree for the Release PR to change, so
#     `trust-tasks-go/trusttasks/version.go` carries a `Version` constant that
#     stands in for the manifest. `publish-go` reads it to decide whether to
#     tag. See the comment on that constant.
#
# There is no registry account and no token: publishing a Go module is pushing
# a tag to a public repository, after which proxy.golang.org serves it.
#
# It is idempotent: the target version is computed from the last TAG, never from
# the branch, so re-running on a fresh checkout recomputes the same answer. It
# exits 0 without touching anything when there is nothing to release; when it
# does bump, it appends one table row to $RELEASE_SUMMARY.
#
# Run from the repository root, on a full-history checkout of `main`, with
# `git-cliff` on PATH.

set -euo pipefail

# Which Go module to release. The core defaults; the nested modules
# (trust-tasks-go/proof, trust-tasks-go/didcomm) pass their own path. Each has
# its own directory, version file, tag prefix and watch set — mirroring
# release-bump-ts.sh and release-bump-dart.sh.
#
# The nested modules are SEPARATE Go modules with their own tags
# (`trust-tasks-go/<name>/v<version>`), so they version independently of the
# core — which is why the core's watch EXCLUDES them: a change under
# trust-tasks-go/proof releases proof, not the core.
PKG="${1:-trust-tasks-go}"
GO_ROOT="github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go"
case "$PKG" in
  trust-tasks-go)
    MODULE="$GO_ROOT"
    PKG_DIR="trust-tasks-go"
    VERSION_FILE="$PKG_DIR/trusttasks/version.go"
    TAG_PREFIX="trust-tasks-go/v"
    # `specs/**` because the packages under `trust-tasks-go/specs` are generated
    # from it, and `scripts/build-go-bindings.mjs` because it is the generator.
    # The nested modules are excluded — they release on their own tags.
    WATCH=("specs" "$PKG_DIR" "scripts/build-go-bindings.mjs"
           ":(exclude)$PKG_DIR/proof" ":(exclude)$PKG_DIR/didcomm" ":(exclude)$PKG_DIR/tsp")
    CLIFF_PATHS=("trust-tasks-go/**" "specs/**" "scripts/build-go-bindings.mjs")
    ;;
  trust-tasks-go/proof)
    MODULE="$GO_ROOT/proof"
    PKG_DIR="trust-tasks-go/proof"
    VERSION_FILE="$PKG_DIR/version.go"
    TAG_PREFIX="trust-tasks-go/proof/v"
    WATCH=("$PKG_DIR")
    CLIFF_PATHS=("$PKG_DIR/**")
    ;;
  trust-tasks-go/didcomm)
    MODULE="$GO_ROOT/didcomm"
    PKG_DIR="trust-tasks-go/didcomm"
    VERSION_FILE="$PKG_DIR/version.go"
    TAG_PREFIX="trust-tasks-go/didcomm/v"
    WATCH=("$PKG_DIR")
    CLIFF_PATHS=("$PKG_DIR/**")
    ;;
  *)
    echo "::error::unknown Go module '$PKG'"
    exit 1
    ;;
esac
CHANGELOG="$PKG_DIR/CHANGELOG.md"

read_version() {
  sed -n 's/^const Version = "\(.*\)"$/\1/p' "$VERSION_FILE"
}

current=$(read_version)
if [ -z "$current" ]; then
  echo "::error::Could not read \`const Version\` from $VERSION_FILE."
  exit 1
fi

# ── Where the last release was ───────────────────────────────────────────────
# `--sort=-version:refname` orders the `<prefix>vX.Y.Z` tags by version rather
# than lexically, so v0.10.0 sorts above v0.9.0. The glob is anchored on the
# module's own prefix, so `trust-tasks-go/v*` never matches a nested module's
# `trust-tasks-go/proof/v*` and vice versa.
tag=$(git tag -l "${TAG_PREFIX}*" --sort=-version:refname | head -1)
if [ -z "$tag" ]; then
  # Reaching here means `publish-go` did not run or did not finish — this job
  # has `needs: publish-go`, and that job writes `${TAG_PREFIX}<Version>` the
  # first time it sees a version with no matching tag. So the module seeds its
  # own anchor and there is nothing to seed by hand.
  #
  # Do NOT tag from a local checkout to clear this. A Go tag IS the publication
  # and cannot be retracted, and tagging locally skips the build/vet/test that
  # `publish-go` runs before it tags. Re-run `publish-go` instead.
  echo "::error::No ${TAG_PREFIX}* tag exists. publish-go writes it — re-run that job rather than tagging by hand; a Go tag is a permanent publication. See RELEASING.md, 'The Go module seeds itself'."
  exit 1
fi
last="${tag#"$TAG_PREFIX"}"

if [ "$current" != "$last" ]; then
  # main already carries a version newer than the last tag: a release is merged
  # but not yet tagged (publish-go failed, or is still running). Proposing
  # another bump on top would release two versions for one set of changes.
  echo "::warning::$VERSION_FILE is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the publish-go job rather than bumping it again."
  exit 0
fi

# ── Is there anything to release? ────────────────────────────────────────────
if git diff --quiet "$tag" HEAD -- "${WATCH[@]}"; then
  echo "::notice::No changes to $MODULE since $tag — nothing to release."
  exit 0
fi

# ── What size of bump? ───────────────────────────────────────────────────────
# Conventional commits, the same signal release-plz reads for the crates: a `!`
# after the type/scope or a `BREAKING CHANGE:` trailer means the compatibility
# field moves; anything else is a patch. As on the npm side there is no
# cargo-semver-checks equivalent to catch an unannounced break, so this is only
# as accurate as the commit subjects. See RELEASING.md.
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
  // Below 1.0 the leading non-zero component is the compatibility boundary, so
  // a break moves the MINOR field and everything else moves the patch — the
  // same rule release-plz applies to the crates and release-bump-ts.sh to the
  // package. 0.0.x is a special case: every 0.0.x is incompatible with every
  // other, so the patch field already IS the boundary and moving the minor
  // would over-bump.
  if (!breaking || (maj === 0 && min === 0)) { console.log(`${maj}.${min}.${pat + 1}`); }
  else if (maj === 0) { console.log(`0.${min + 1}.0`); }
  else { console.log(`${maj + 1}.0.0`); }
')

# A Go module at v2 or above must carry the major in its module path
# (`.../trust-tasks-go/v2`), and `go.mod` must be edited to match or every
# `go get` of the new version fails with a "module path mismatch" that no amount
# of retagging fixes. That is a deliberate, human migration — refuse to propose
# it automatically.
next_major="${next%%.*}"
if [ "$next_major" -ge 2 ]; then
  echo "::error::This bump would produce v$next, and Go requires the module path to end in /v$next_major from v2 onward. Update the \`module\` line in $PKG_DIR/go.mod, every import path in the tree, and MODULE_PATH in scripts/build-go-bindings.mjs, then tag by hand. See RELEASING.md."
  exit 1
fi

echo "::notice::$MODULE $last -> $next ($level)"

# ── Apply the bump ───────────────────────────────────────────────────────────
# In the working tree only. scripts/release-pr.sh commits every package's bump
# together, as the one Release PR.

NEXT="$next" node -e '
  const fs = require("fs");
  const p = process.argv[1], v = process.env.NEXT;
  const raw = fs.readFileSync(p, "utf8");
  // Rewrite only the constant, leaving the doc comment above it — which
  // explains why the constant exists — untouched.
  const out = raw.replace(/^(const Version = ")[^"]*(")$/m, `$1${v}$2`);
  if (out === raw) { console.error(`could not rewrite const Version in ${p}`); process.exit(1); }
  fs.writeFileSync(p, out);
' "$VERSION_FILE"

# Changelog generation is best-effort ON PURPOSE. The version bump is the
# load-bearing part of this PR — it is what decides the tag — and a git-cliff
# hiccup must not be able to block a release. A missing section is visible in
# the PR diff and can be written by hand.
cliff_include=()
for p in "${CLIFF_PATHS[@]}"; do
  cliff_include+=(--include-path "$p")
done
if section=$(git-cliff --config cliff.toml \
  --strip header \
  --tag "v$next" \
  --unreleased \
  "${cliff_include[@]}" \
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
printf '| `%s` | Go (tag `%s`) | `%s` → `%s` | %s |\n' "$MODULE" "${TAG_PREFIX}$next" "$last" "$next" "$level" >>"${RELEASE_SUMMARY:-/dev/stdout}"
