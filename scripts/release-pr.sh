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
#      `core` version group (release-plz.toml). Bump levels come from the
#      conventional commits; an undeclared API break was already refused on its
#      pull request (`rs API compatibility`). Only the PR-opening half of release-plz is dropped.
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
# Two phases, so that the release token is never present while third-party
# code runs:
#
#   release-pr.sh prepare DIR   steps 1 and 2. Runs release-plz, git-cliff and
#                               the bump scripts; needs no token. Writes the
#                               change as DIR/changes.patch and the release
#                               table as DIR/summary.md.
#   release-pr.sh open DIR      step 3. Applies DIR/changes.patch to the same
#                               commit, commits it and pushes the branch and the
#                               PR. Runs only git, gh, jq and sed, and refuses a
#                               patch outside scripts/check-patch-paths.sh's
#                               release allowlist before applying it.
#   release-pr.sh               both, one after the other (by hand).
#
# publish.yml runs the two phases in separate jobs, `release-pr` and
# `release-pr-open`; only the second has the token.
#
# Run from the repository root, on a full-history checkout of `main`, with
# `release-plz`, `git-cliff`, `node` and `jq` on PATH for `prepare`, and `gh`
# authenticated (GH_TOKEN) for `open`. When PUSH_TOKEN is set, `open` pushes
# with it; otherwise with git's own credentials.

set -Eeuo pipefail
# Name the command that failed. Under `set -e` a failing step otherwise ends the
# job with a bare exit code (141 is SIGPIPE) and no line to look at; -E carries
# the trap into functions and subshells.
trap 'rc=$?; echo "::error file=${BASH_SOURCE[0]},line=${LINENO}::exit ${rc} from: ${BASH_COMMAND}" >&2' ERR

usage() {
  echo "usage: $0 [prepare DIR | open DIR]" >&2
  exit 2
}

BRANCH="release"

# The non-Rust packages, in release order. The crates need no list — release-plz
# reads the workspace, and `publish = false` in a manifest keeps a crate out.
#
# Adding a package means an entry here AND its matrix leg in publish.yml
# (publish-npm / publish-go / tag-dart). `npm run check-dart-packages` compares
# DART_PACKAGES against the other Dart lists, and `npm run check-release-lists`
# TS_PACKAGES and GO_MODULES against theirs; each list's first entry must stay
# the core, as in the publish matrices.
TS_PACKAGES=(
  "@openvtc/trust-tasks"
  "@openvtc/trust-tasks-proof"
  "@openvtc/trust-tasks-tsp"
  "@openvtc/trust-tasks-capability-client"
  "@openvtc/trust-tasks-gen"
)
GO_MODULES=(
  "trust-tasks-go"
  "trust-tasks-go/proof"
  "trust-tasks-go/didcomm"
  "trust-tasks-go/capabilityclient"
  "trust-tasks-go/specs"
)
DART_PACKAGES=(
  "trust_tasks"
  "trust_tasks_proof"
  "trust_tasks_https"
  "trust_tasks_didcomm"
  "trust_tasks_capability_client"
)

# ── prepare: compute the release (no token) ─────────────────────────────────
prepare() {
  local out="$1"
  mkdir -p "$out"
  if [ -n "$(git status --porcelain)" ]; then
    echo "::error::prepare needs a clean checkout" >&2
    exit 1
  fi

  RELEASE_SUMMARY="$out/summary.md"
  : >"$RELEASE_SUMMARY"
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
  # release-plz.toml (the `core` version group, cliff.toml) the same
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
  # `trust_tasks` is DART_PACKAGES' first entry, so its bump lands before the
  # siblings', and they read its new version: one whose range no longer admits
  # it (the core crossed a 0.x minor) has its floor raised and is released for
  # it. check-sibling-core-ranges fails the Release PR otherwise (#1085).
  for pkg in "${DART_PACKAGES[@]}"; do
    TRUST_TASKS_CORE=$(sed -n 's/^version: *//p' trust-tasks-dart/pubspec.yaml | sed -n 1p | tr -d '\r') \
      ./scripts/release-bump-dart.sh "$pkg"
  done
  # A sibling that is tested but not released yet (trust-tasks-dart-tsp) is held
  # to the same check, so its floor moves too; it has no version to bump.
  dart_core=$(sed -n 's/^version: *//p' trust-tasks-dart/pubspec.yaml | sed -n 1p | tr -d '\r')
  for pubspec in trust-tasks-dart-*/pubspec.yaml; do
    was=$(node scripts/raise-dart-core-floor.mjs "$pubspec" "$dart_core")
    [ -z "$was" ] || echo "::notice::$pubspec: trust_tasks $was -> ^$dart_core"
  done

  # Everything the steps above changed, as one patch against HEAD, and the
  # commit it applies to.
  git add -A
  git diff --cached --binary >"$out/changes.patch"
  git rev-parse HEAD >"$out/base"
  # Leave the checkout as it was: `open` applies the patch to it.
  git reset -q --hard
  git clean -fdq
}

# ── open: commit, push and open the PR (the token, and nothing else) ─────────

# `git push "$@"`, authenticated with $PUSH_TOKEN when it is set (the release
# workflow), or with git's own credentials when it is not (by hand). Defined
# here rather than calling scripts/git-push-with-token.sh, because by the time
# `open` pushes, the working tree is the patch's, and nothing from it may run
# while the token is in the environment.
# ── What a release may change inside a manifest ──────────────────────────────
# check-patch-paths.sh limits WHICH files the patch touches; these limit WHAT
# it changes in the three kinds of manifest a release edits, so a version bump
# cannot carry anything else with it: an npm `scripts` entry, a dependency
# moved to a path, URL or git source, a line of Go or Dart beside the version
# constant, or a pubspec key other than `version:` and a dependency constraint. They read the
# index after `git apply --index` and HEAD, with git, jq and grep only.

# A version, or a range of versions, and nothing else.
VERSION_RANGE_RE='^([\^~]|[<>]=?|=)?[0-9]+(\.[0-9]+){0,2}(-[0-9A-Za-z.]+)?( +([<>]=?|=)?[0-9]+(\.[0-9]+){0,2}(-[0-9A-Za-z.]+)?)*$'

refuse_content() {
  echo "::error::refusing the release change: $*" >&2
}

# The lines a staged change removes and adds, without the file headers.
changed_lines() {
  git diff --cached -U0 -- "$1" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---) ' || true
}

# Only `version` and dependency version strings may differ.
check_package_json() {
  local file="$1" old new filter
  if ! old=$(git show "HEAD:$file" 2>/dev/null); then
    refuse_content "$file is new; a release only edits existing manifests"
    return 1
  fi
  new=$(git show ":$file")
  filter='del(.version)
    | reduce ("dependencies", "devDependencies", "peerDependencies", "optionalDependencies") as $k
        (.; if (.[$k] | type) == "object"
            then .[$k] |= with_entries(
              if (.value | type) == "string" and (.value | test($re)) then .value = "<version>" else . end)
            else . end)'
  if ! old=$(jq -S --arg re "$VERSION_RANGE_RE" "$filter" <<<"$old") \
    || ! new=$(jq -S --arg re "$VERSION_RANGE_RE" "$filter" <<<"$new"); then
    refuse_content "$file is not valid JSON"
    return 1
  fi
  if [ "$old" != "$new" ]; then
    refuse_content "$file changes more than its version and dependency versions"
    return 1
  fi
}

# Only the one version-constant line may change, to another version: Go's
# `const Version` and Dart's `packageVersion` (the pubspec's mirror).
check_version_constant() {
  local file="$1" re="$2" what="$3" line n=0
  while IFS= read -r line; do
    n=$((n + 1))
    if ! [[ "$line" =~ $re ]]; then
      refuse_content "$file changes more than \`$what\`: $line"
      return 1
    fi
  done < <(changed_lines "$file")
  if [ "$n" -ne 2 ]; then
    refuse_content "$file should change exactly one \`$what\` line"
    return 1
  fi
}
GO_VERSION_RE='^[+-]const Version = "[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?"$'
DART_VERSION_RE="^[+-]const String packageVersion = '[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?';\$"

# Only `version:` and dependency constraints may change, each line in place:
# the same key, at the same indentation, from one version constraint to another.
check_pubspec() {
  local file="$1" line i
  local -a removed=() added=()
  while IFS= read -r line; do
    case "$line" in
      -*) removed+=("${line#-}") ;;
      +*) added+=("${line#+}") ;;
    esac
  done < <(changed_lines "$file")
  if [ "${#removed[@]}" -ne "${#added[@]}" ]; then
    refuse_content "$file adds or removes lines; a release only changes versions in place"
    return 1
  fi
  local re='^( *)([a-z_][a-z0-9_]*): *(.*)$' key_old key_new value
  for i in "${!added[@]}"; do
    for line in "${removed[$i]}" "${added[$i]}"; do
      if ! [[ "$line" =~ $re ]]; then
        refuse_content "$file: not a version line: $line"
        return 1
      fi
      if [ -z "${BASH_REMATCH[1]}" ] && [ "${BASH_REMATCH[2]}" != "version" ]; then
        refuse_content "$file: only \`version:\` may change at the top level: $line"
        return 1
      fi
      value="${BASH_REMATCH[3]}"
      value="${value#[\"\']}"
      value="${value%[\"\']}"
      if ! [[ "$value" =~ $VERSION_RANGE_RE ]]; then
        refuse_content "$file: not a version constraint: $line"
        return 1
      fi
    done
    key_old="${removed[$i]%%:*}"
    key_new="${added[$i]%%:*}"
    if [ "$key_old" != "$key_new" ]; then
      refuse_content "$file: \`$key_old\` became \`$key_new\`"
      return 1
    fi
  done
}

check_applied_content() {
  local file bad=0
  while IFS= read -r -d '' file; do
    case "$file" in
      */package.json) check_package_json "$file" || bad=1 ;;
      */version.go) check_version_constant "$file" "$GO_VERSION_RE" "const Version" || bad=1 ;;
      */lib/src/runtime/version.dart) check_version_constant "$file" "$DART_VERSION_RE" "packageVersion" || bad=1 ;;
      */pubspec.yaml) check_pubspec "$file" || bad=1 ;;
    esac
  done < <(git diff --cached --name-only -z)
  [ "$bad" -eq 0 ]
}

push_branch() {
  { set +x; } 2>/dev/null
  if [ -z "${PUSH_TOKEN:-}" ]; then
    git push "$@"
    return
  fi
  local auth
  auth=$(printf 'x-access-token:%s' "$PUSH_TOKEN" | base64 | tr -d '\n')
  echo "::add-mask::$auth"
  GIT_CONFIG_COUNT=1 \
    GIT_CONFIG_KEY_0="http.https://github.com/.extraheader" \
    GIT_CONFIG_VALUE_0="AUTHORIZATION: basic $auth" \
    git push "$@"
}

open_pr() {
  local out="$1"
  if [ "$(git rev-parse HEAD)" != "$(cat "$out/base")" ]; then
    echo "::error::the release change was computed against $(cat "$out/base"), not $(git rev-parse HEAD)" >&2
    exit 1
  fi
  # Before anything else, and from this checkout as it stands: the patch may
  # touch only the files a release changes (scripts/check-patch-paths.sh).
  # After `git apply` below, nothing from the working tree runs.
  ./scripts/check-patch-paths.sh release "$out/changes.patch"

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

  existing=$(gh pr list --head "$BRANCH" --state open --json number --jq '.[0].number // empty')

  # Always rebuild the branch from the current main. The branch is a derived
  # artefact; nothing on it is worth preserving across runs.
  git switch -C "$BRANCH"
  if [ -s "$out/changes.patch" ]; then
    git apply --index --binary "$out/changes.patch"
    # Before the commit, so a refused change is never pushed.
    check_applied_content || exit 1
  fi
  RELEASE_SUMMARY="$out/summary.md"

  if git diff --cached --quiet; then
    echo "::notice::Nothing to release."
    if [ -n "$existing" ]; then
      # Only reachable when main moved backwards (a revert) after the PR opened —
      # the normal end of a Release PR is being merged.
      gh pr close "$existing" --comment "Nothing is left to release on \`main\`; closing. The next releasable merge reopens a Release PR."
    fi
    return 0
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
  push_branch --force origin "$BRANCH"

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
  the commit subjects.
- **npm** — \`publish-npm\` publishes with OIDC provenance and tags
  \`<prefix>-v<version>\`.
- **Go** — \`verify-go\` builds, vets and tests, then \`publish-go\` pushes the
  tag, which *is* the publication. **Irreversible**: proxy.golang.org never forgets a tag.
- **pub.dev** — \`verify-dart\` checks each package against the *published* core,
  \`tag-dart\` pushes the tag, and \`publish-dart.yml\` publishes from it.

Every bump follows the commit subjects; an undeclared API break is refused on its
pull request by that language's API compatibility check (\`rs\`, \`ts\`, \`go\` and
\`Dart API compatibility\`).

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
}

case "${1:-}" in
  prepare) [ $# -eq 2 ] || usage; prepare "$2" ;;
  open) [ $# -eq 2 ] || usage; open_pr "$2" ;;
  "")
    dir="$(mktemp -d)"
    prepare "$dir"
    open_pr "$dir"
    ;;
  *) usage ;;
esac
