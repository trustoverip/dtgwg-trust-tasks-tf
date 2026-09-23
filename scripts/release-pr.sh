#!/usr/bin/env bash
#
# Opens or updates THE Release PR — one PR carrying the next release of every
# package in this repo: the Rust crates, the npm packages, the Go modules and
# the Dart packages.
#
# Why one: this used to be up to thirteen PRs (release-plz's for the crates,
# plus one per npm package, Go module and Dart package). Merging any of them is
# a push to `main`, and every push regenerated every other Release PR from the
# new `main` and restarted its CI — so cutting a release meant merge, wait for
# twelve CI runs, merge the next, wait again. One PR makes a release one merge.
#
# How it is built, on a fresh branch from `main`:
#
#   1. `release-plz update` bumps the crates and writes their changelogs —
#      everything the release-plz Release PR used to contain, including the
#      cargo-semver-checks bump levels and the `core` version group
#      (release-plz.toml). Only the PR-opening half of release-plz is dropped.
#   2. scripts/release-bump-{ts,go,dart}.sh bump each npm package, Go module
#      and Dart package from the conventional commits since its own tag. Each
#      is a no-op when its package has nothing to release.
#   3. Whatever changed is committed once, signed off, and force-pushed to the
#      `release` branch; the PR is opened or retitled to match.
#
# Merging it is the release, exactly as before, because every publish job in
# publish.yml keys on "this version is not published yet", not on which PR
# merged: `release-plz-release` for the crates, `publish-npm`, `publish-go`, and
# `tag-dart` → publish-dart.yml. Packages keep their own versions and tags —
# only the PR is shared.
#
# Idempotent: every bump is computed from registry state and tags, never from
# the branch, so a re-run recomputes the same tree.
#
# Run from the repository root, on a full-history checkout of `main`, with `gh`
# authenticated (GH_TOKEN), and `release-plz`, `cargo-semver-checks`,
# `git-cliff`, `node` and `jq` on PATH.

set -euo pipefail

BRANCH="release"

# The non-Rust packages, in release order. The crates need no list — release-plz
# reads the workspace, and `publish = false` in a manifest keeps a crate out.
#
# Adding a package means an entry here AND its matrix leg in publish.yml
# (publish-npm / publish-go / tag-dart). `npm run check-dart-packages` compares
# DART_PACKAGES against the other Dart lists; its first entry must stay
# `trust_tasks`, as in tag-dart.
TS_PACKAGES=(
  "@openvtc/trust-tasks"
  "@openvtc/trust-tasks-proof"
  "@openvtc/trust-tasks-tsp"
  "@openvtc/trust-tasks-capability-client"
)
GO_MODULES=(
  "trust-tasks-go"
  "trust-tasks-go/proof"
  "trust-tasks-go/didcomm"
)
DART_PACKAGES=(
  "trust_tasks"
  "trust_tasks_proof"
  "trust_tasks_https"
  "trust_tasks_didcomm"
  "trust_tasks_capability_client"
)

# ── Commit identity ───────────────────────────────────────────────────────────
# Author as the identity behind the release token, not as the bot. EasyCLA
# authorises the commit *author*, and `github-actions[bot]` has signed no CLA —
# so a bot-authored release commit fails EasyCLA however well it is signed off.
#
# Resolved from the token rather than hard-coded: whoever the token belongs to
# is who the release is attributable to, and hard-coding a person here would
# quietly misattribute it the moment the token changed hands.
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

RELEASE_SUMMARY="$(mktemp)"
export RELEASE_SUMMARY

# ── Rust ─────────────────────────────────────────────────────────────────────
# The published crates and their versions, before and after release-plz.
# `publish: []` is how cargo metadata spells `publish = false`.
crate_versions() {
  cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.publish != []) | "\(.name) \(.version)"' \
    | sort
}
before="$(crate_versions)"

# release-plz insists on a clean tree, so it runs first. It reads
# release-plz.toml (semver_check, the `core` version group, cliff.toml) the same
# way `release-pr` did.
release-plz update

after="$(crate_versions)"
# Level from the versions themselves: below 1.0 the leading non-zero component
# is the compatibility boundary, the same rule the bump scripts apply.
join <(echo "$before") <(echo "$after") | while read -r crate old new; do
  [ "$old" = "$new" ] && continue
  level=$(OLD="$old" NEW="$new" node -e '
    const lead = (v) => { const p = v.split(".").map(Number);
      return p[0] ? `${p[0]}` : p[1] ? `0.${p[1]}` : `0.0.${p[2]}`; };
    console.log(lead(process.env.OLD) === lead(process.env.NEW) ? "patch" : "breaking");')
  printf '| `%s` | crates.io | `%s` → `%s` | %s |\n' "$crate" "$old" "$new" "$level" >>"$RELEASE_SUMMARY"
done

# ── npm, Go, Dart ────────────────────────────────────────────────────────────
for pkg in "${TS_PACKAGES[@]}"; do ./scripts/release-bump-ts.sh "$pkg"; done
for mod in "${GO_MODULES[@]}"; do ./scripts/release-bump-go.sh "$mod"; done
for pkg in "${DART_PACKAGES[@]}"; do ./scripts/release-bump-dart.sh "$pkg"; done

existing=$(gh pr list --head "$BRANCH" --state open --json number --jq '.[0].number // empty')

git add -A
if git diff --cached --quiet; then
  echo "::notice::Nothing to release."
  if [ -n "$existing" ]; then
    # Only reachable when main moved backwards (a revert) after the PR opened —
    # the normal end of a Release PR is being merged.
    gh pr close "$existing" --comment "Nothing is left to release on \`main\`; closing. The next releasable merge reopens a Release PR."
  fi
  exit 0
fi

count=$(grep -c '^|' "$RELEASE_SUMMARY" || true)
if [ "$count" -eq 1 ]; then
  # `| \`name\` | registry | \`a\` → \`b\` | level |` → "name b"
  title="chore: release $(sed -E 's/^\| `([^`]*)` \|[^|]*\| `[^`]*` → `([^`]*)`.*/\1 \2/' "$RELEASE_SUMMARY")"
else
  title="chore: release $count packages"
fi
# `chore: release` is skipped by cliff.toml, so the merge commit never appears in
# a changelog of its own.

table_file="$(mktemp)"
{
  echo "| Package | Registry | Version | Bump |"
  echo "|---|---|---|---|"
  cat "$RELEASE_SUMMARY"
} >"$table_file"

# -s: DCO sign-off is mandatory on every commit in this repo.
git commit -q -s -m "$title" -m "$(cat "$table_file")" -m \
  "Automated by scripts/release-pr.sh. Merging this PR publishes every package listed."
git push --force origin "$BRANCH"

# ── Open or refresh the PR ───────────────────────────────────────────────────
# Written to a file rather than captured in `$(cat <<EOF)`: bash scans a
# command substitution for quote balance, and an apostrophe in the heredoc body
# makes it read to end-of-file looking for a closing quote.
body_file="$(mktemp)"
cat >"$body_file" <<EOF
The Release PR for every package in this repo. **Merging this is the release.**

$(cat "$table_file")

What merging does, per registry (all in \`publish.yml\`, each keyed on "this
version is not published yet"):

- **crates.io** — \`release-plz-release\` tags \`<crate>-v<version>\`, publishes in
  dependency order and creates a GitHub Release per crate. Bump levels come from
  \`cargo-semver-checks\`.
- **npm** — \`publish-npm\` publishes with OIDC provenance and tags
  \`<prefix>-v<version>\`.
- **Go** — \`publish-go\` builds, vets and tests, then pushes the tag, which *is*
  the publication. **Irreversible**: proxy.golang.org never forgets a tag.
- **pub.dev** — \`tag-dart\` verifies against the *published* core and pushes the
  tag; \`publish-dart.yml\` publishes from it.

The npm, Go and Dart bumps are only as accurate as the commit subjects — nothing
checks their APIs the way \`cargo-semver-checks\` checks the crates. Look for a
missing \`!\` before merging.

This branch is regenerated from \`main\` on every push, so do not commit to it —
edits are force-pushed away. See RELEASING.md.
EOF

if [ -n "$existing" ]; then
  # REST, not `gh pr edit`. `gh pr edit` resolves editable metadata before it
  # writes — assignees, reviewers, milestones and organization TEAMS — so it
  # asks GraphQL for fields that require `read:org`. The release token has
  # `repo` only, by design, so `gh pr edit` failed the job *after* the branch
  # had been force-pushed, leaving a PR whose title and body described the
  # previous release while the branch described the new one.
  gh api --silent -X PATCH "repos/{owner}/{repo}/pulls/$existing" \
    -f title="$title" -f body="$(cat "$body_file")"
  echo "::notice::updated PR #$existing"
else
  gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file" --label release \
    || gh pr create --base main --head "$BRANCH" --title "$title" --body-file "$body_file"
fi
