#!/usr/bin/env bash
#
# Opens or updates the Release PR for the `trust-tasks-go` module.
#
# The third of three. release-plz keeps the Release PR for the Rust crates;
# scripts/release-ts-pr.sh does it for `@openvtc/trust-tasks`; this does it for
# the Go module. All three work the same way so every half of a release reads
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
# the branch, so re-running recomputes the same answer and force-pushes an
# identical tree.
#
# Run from the repository root, on a full-history checkout of `main`, with `gh`
# authenticated (GH_TOKEN) and `git-cliff` on PATH.

set -euo pipefail

# Which Go module to release. The core defaults; the nested modules
# (trust-tasks-go/proof, trust-tasks-go/didcomm) pass their own path. Each has
# its own directory, version file, release branch, tag prefix and watch set —
# mirroring release-ts-pr.sh and release-dart-pr.sh.
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
    BRANCH="release-go"
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
    BRANCH="release-go-proof"
    TAG_PREFIX="trust-tasks-go/proof/v"
    WATCH=("$PKG_DIR")
    CLIFF_PATHS=("$PKG_DIR/**")
    ;;
  trust-tasks-go/didcomm)
    MODULE="$GO_ROOT/didcomm"
    PKG_DIR="trust-tasks-go/didcomm"
    VERSION_FILE="$PKG_DIR/version.go"
    BRANCH="release-go-didcomm"
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
  echo "::warning::$VERSION_FILE is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the publish-go job rather than opening another Release PR."
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
if git log --format='%s%n%b' "$range" -- "${WATCH[@]}" \
  | grep -qE '^[a-z]+(\([^)]*\))?!:|^BREAKING[ -]CHANGE'; then
  level=breaking
else
  level=patch
fi

next=$(LEVEL="$level" CURRENT="$last" node -e '
  const [maj, min, pat] = process.env.CURRENT.split(".").map(Number);
  const breaking = process.env.LEVEL === "breaking";
  // Below 1.0 the leading non-zero component is the compatibility boundary, so
  // a break moves the MINOR field and everything else moves the patch — the
  // same rule release-plz applies to the crates and release-ts-pr.sh to the
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

# ── Build the release commit ─────────────────────────────────────────────────
# Author as the identity behind the release token, not as the bot: EasyCLA
# authorises the commit *author*, and `github-actions[bot]` has signed no CLA.
# Same reasoning as release-ts-pr.sh, at more length there.
if ! author_json=$(gh api user 2>/dev/null); then
  echo "::error::Could not resolve the token owner via \`gh api user\`. A GitHub App token has no user, so this needs an explicit identity — do NOT fall back to github-actions[bot], which fails EasyCLA silently."
  exit 1
fi
author_login=$(jq -r '.login' <<<"$author_json")
author_id=$(jq -r '.id' <<<"$author_json")
author_name=$(jq -r '.name // .login' <<<"$author_json")
git config user.name "$author_name"
git config user.email "${author_id}+${author_login}@users.noreply.github.com"

# Always rebuild the branch from the current main. The branch is a derived
# artefact; nothing on it is worth preserving across runs.
git switch -C "$BRANCH"

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

if git diff --quiet; then
  echo "::notice::nothing changed — leaving the branch alone."
  exit 0
fi

git add "$VERSION_FILE" "$CHANGELOG"
# -s: DCO sign-off is mandatory on every commit in this repo.
git commit -s -m "chore: release $MODULE $next" -m \
  "Automated by scripts/release-go-pr.sh. Merging this PR tags ${TAG_PREFIX}$next, which is what publishes the module."
git push --force origin "$BRANCH"

# ── Open or refresh the PR ───────────────────────────────────────────────────
title="chore: release $PKG $next"
watched=$(printf '`%s`, ' "${WATCH[@]}"); watched="${watched%, }"
body_file="$(mktemp)"
cat >"$body_file" <<EOF
Release PR for the \`$PKG\` Go module, the counterpart to the release-plz PR for
the crates and the \`release-ts\` PR for the npm package. **Merging this is the
release**: the \`publish-go\` job in \`publish.yml\` sees a \`Version\` that has
no matching tag and pushes \`${TAG_PREFIX}$next\`, after which
\`proxy.golang.org\` serves it and \`go get $MODULE@v$next\` resolves.

- \`$MODULE\`: \`$last\` → \`$next\` (\`$level\`)
- derived from the conventional commits in \`$range\` touching $watched

There is no registry account and no token involved — a Go module is published
by tagging a public repository.

This branch is regenerated from \`main\` on every push, so do not commit to it —
edits are force-pushed away. See RELEASING.md.
EOF

existing=$(gh pr list --head "$BRANCH" --state open --json number --jq '.[0].number // empty')
if [ -n "$existing" ]; then
  # REST, not `gh pr edit` — the latter resolves editable metadata (assignees,
  # reviewers, org teams) and so needs `read:org`, which the release token does
  # not have. See the long note in release-ts-pr.sh.
  gh api --silent -X PATCH "repos/{owner}/{repo}/pulls/$existing" \
    -f title="$title" -f body="$(cat "$body_file")"
  echo "::notice::updated PR #$existing"
else
  gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file" --label release \
    || gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file"
fi
