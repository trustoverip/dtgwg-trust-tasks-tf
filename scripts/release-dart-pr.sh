#!/usr/bin/env bash
#
# Opens or updates the Release PR for one Dart package.
#
#   scripts/release-dart-pr.sh [trust_tasks|trust_tasks_proof|trust_tasks_https]
#
# Defaults to `trust_tasks`. Each package has its own tag prefix, its own
# release branch and its own PR, because each is published to pub.dev on its
# own schedule; the package table below is the single place they differ.
#
# The fourth of four. release-plz keeps the Release PR for the Rust crates;
# scripts/release-ts-pr.sh does it for `@openvtc/trust-tasks`;
# scripts/release-go-pr.sh for the Go module; this for the Dart package. All
# four work the same way so every part of a release reads identically — see
# RELEASING.md.
#
# ── What is different about Dart ─────────────────────────────────────────────
#
# pub.dev only accepts an automated publish when the workflow run was triggered
# by pushing a git TAG, which it verifies in the GitHub OIDC token's claims. So
# unlike the npm side — where merging the Release PR publishes directly — merging
# this PR makes `tag-dart` write `trust-tasks-dart-v<version>`, and that tag
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
# the branch, so re-running recomputes the same answer and force-pushes an
# identical tree.
#
# Run from the repository root, on a full-history checkout of `main`, with `gh`
# authenticated (GH_TOKEN) and `git-cliff` on PATH.

set -euo pipefail

PKG="${1:-trust_tasks}"

# WATCH is everything that can change what the package publishes.
case "$PKG" in
  trust_tasks)
    PKG_DIR="trust-tasks-dart"
    BRANCH="release-dart"
    VERSION_FILE="$PKG_DIR/lib/src/runtime/version.dart"
    # `specs/**` because the libraries under `trust-tasks-dart/lib/specs` are
    # generated from it, and `scripts/build-dart-bindings.mjs` because it is the
    # generator.
    WATCH=("specs" "$PKG_DIR" "scripts/build-dart-bindings.mjs")
    ;;
  trust_tasks_proof)
    PKG_DIR="trust-tasks-dart-proof"
    BRANCH="release-dart-proof"
    VERSION_FILE=""
    # Only its own tree. A core change that alters this package's behaviour
    # reaches consumers through the core's release, not this one — the same
    # separation as between trust-tasks-rs and trust-tasks-proof.
    WATCH=("$PKG_DIR")
    ;;
  trust_tasks_https)
    PKG_DIR="trust-tasks-dart-https"
    BRANCH="release-dart-https"
    VERSION_FILE=""
    # Only its own tree, for the same reason as trust_tasks_proof.
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
  echo "::warning::$PUBSPEC is at $current but the newest tag is $tag. A release is staged and unpublished — re-run the tag-dart job rather than opening another Release PR."
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
if git log --format='%s%n%b' "$range" -- "${WATCH[@]}" \
  | grep -qE '^[a-z]+(\([^)]*\))?!:|^BREAKING[ -]CHANGE'; then
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

if git diff --quiet; then
  echo "::notice::nothing changed — leaving the branch alone."
  exit 0
fi

git add "$PUBSPEC" "$CHANGELOG" ${VERSION_FILE:+"$VERSION_FILE"}
# -s: DCO sign-off is mandatory on every commit in this repo.
git commit -s -m "chore: release $PKG $next" -m \
  "Automated by scripts/release-dart-pr.sh. Merging this PR tags $TAG_PREFIX$next, and that tag is what publishes the package to pub.dev."
git push --force origin "$BRANCH"

# ── Open or refresh the PR ───────────────────────────────────────────────────
title="chore: release $PKG $next"
watched=$(printf '`%s`, ' "${WATCH[@]}"); watched=${watched%, }
body_file="$(mktemp)"
cat >"$body_file" <<EOF
Release PR for the Dart package \`$PKG\`, the counterpart to the release-plz PR for the
crates and the \`release-ts\` / \`release-go\` PRs. **Merging this starts the
release**: the \`tag-dart\` job in \`publish.yml\` verifies the package and pushes
\`$TAG_PREFIX$next\`, and that tag triggers \`publish-dart.yml\`, which
publishes to pub.dev over OIDC.

- \`$PKG\`: \`$last\` → \`$next\` (\`$level\`)
- derived from the conventional commits in \`$range\` touching $watched

Unlike the other three, pub.dev refuses a publish that was not triggered by a tag
push — so the tag is a step in the chain rather than a record of one. If
\`RELEASE_PLZ_TOKEN\` is unset the tag is pushed by the default token, which does
not trigger workflows; \`tag-dart\` warns when that happens and the fix is to
re-push the tag from a workstation.

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
