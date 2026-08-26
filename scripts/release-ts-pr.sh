#!/usr/bin/env bash
#
# Opens or updates the Release PR for `@openvtc/trust-tasks`.
#
# release-plz does this for the nine Rust crates. It cannot do it for the npm
# package: release-plz is Rust-only, has no pre/post hooks, and will not touch a
# non-Rust manifest, so there is no way to make it bump `package.json` inside
# the crates Release PR. This script is the npm half, deliberately built to work
# the same way so both halves of a release read identically:
#
#   * the previous release is a tag  — `trust-tasks-ts-v<version>`, written by
#     the `publish-npm` job, mirroring release-plz's `<crate>-v<version>`;
#   * the bump comes from conventional commits since that tag, over the paths
#     that can change what the package publishes;
#   * the changelog section comes from git-cliff with the same `cliff.toml`
#     release-plz uses;
#   * merging the PR is the release — `publish-npm` sees a version that is not
#     on npm and publishes it.
#
# It is idempotent: the target version is computed from the last TAG, never
# from the branch, so re-running recomputes the same answer and force-pushes an
# identical tree.
#
# Run from the repository root, on a full-history checkout of `main`, with
# `gh` authenticated (GH_TOKEN) and `git-cliff` on PATH.

set -euo pipefail

PKG_DIR="trust-tasks-ts"
MANIFEST="$PKG_DIR/package.json"
CHANGELOG="$PKG_DIR/CHANGELOG.md"
BRANCH="release-ts"

# Everything that can change what this package publishes. `specs/**` is here
# because the bindings under `trust-tasks-ts/src` are generated from it, and
# `scripts/build-ts-bindings.mjs` because it is the generator.
WATCH=("specs" "$PKG_DIR" "scripts/build-ts-bindings.mjs")

name=$(node -p "require('./$MANIFEST').name")
current=$(node -p "require('./$MANIFEST').version")

# ── Where the last release was ───────────────────────────────────────────────
tag=$(git tag -l 'trust-tasks-ts-v*' --sort=-version:refname | head -1)
if [ -z "$tag" ]; then
  echo "::error::No trust-tasks-ts-v* tag exists, so there is no anchor to measure this release from. Seed it once at the currently-published version — see RELEASING.md, 'One-time migration'."
  exit 1
fi
last="${tag#trust-tasks-ts-v}"

if [ "$current" != "$last" ]; then
  # main already carries a version newer than the last tag: a release is merged
  # but not yet on npm (publish-npm failed, or is still running). Proposing
  # another bump on top would release two versions for one set of changes.
  echo "::warning::$MANIFEST is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the publish-npm job rather than opening another Release PR."
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
if git log --format='%s%n%b' "$range" -- "${WATCH[@]}" \
  | grep -qE '^[a-z]+(\([^)]*\))?!:|^BREAKING[ -]CHANGE'; then
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

# ── Build the release commit ─────────────────────────────────────────────────
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

# Always rebuild the branch from the current main. The branch is a derived
# artefact; nothing on it is worth preserving across runs.
git switch -C "$BRANCH"

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
if section=$(git-cliff --config cliff.toml \
  --strip header \
  --tag "v$next" \
  --unreleased \
  --include-path 'trust-tasks-ts/**' \
  --include-path 'specs/**' \
  --include-path 'scripts/build-ts-bindings.mjs' \
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

git add "$MANIFEST" "$CHANGELOG" "$PKG_DIR/package-lock.json" 2>/dev/null || git add "$MANIFEST" "$CHANGELOG"
# -s: DCO sign-off is mandatory on every commit in this repo.
git commit -s -m "chore: release $name $next" -m \
  "Automated by scripts/release-ts-pr.sh. Merging this PR publishes $name $next to npm."
git push --force origin "$BRANCH"

# ── Open or refresh the PR ───────────────────────────────────────────────────
title="chore: release $name $next"
# Written to a file rather than captured in `$(cat <<EOF)`: bash scans a
# command substitution for quote balance, and an apostrophe in the heredoc body
# ("publish.yml\'s") makes it read to end-of-file looking for a closing quote.
body_file="$(mktemp)"
cat >"$body_file" <<EOF
Release PR for the npm package, the counterpart to the release-plz PR for the
crates. **Merging this is the release**: the \`publish-npm\` job in
\`publish.yml\` sees a version that is not on npm, publishes it with OIDC
provenance, and tags \`trust-tasks-ts-v$next\`.

- \`$name\`: \`$last\` → \`$next\` (\`$level\`)
- derived from the conventional commits in \`$range\` touching \`specs/\`,
  \`trust-tasks-ts/\` or \`scripts/build-ts-bindings.mjs\`

This branch is regenerated from \`main\` on every push, so do not commit to it —
edits are force-pushed away. See RELEASING.md.
EOF

existing=$(gh pr list --head "$BRANCH" --state open --json number --jq '.[0].number // empty')
if [ -n "$existing" ]; then
  gh pr edit "$existing" --title "$title" --body-file "$body_file"
  echo "::notice::updated PR #$existing"
else
  gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file" --label release \
    || gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file"
fi
