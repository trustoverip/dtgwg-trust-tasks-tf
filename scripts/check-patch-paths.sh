#!/usr/bin/env bash
# check-patch-paths.sh <release|withdraw> PATCH
#
# Refuses a patch that touches anything its producer has no reason to touch.
#
# The Release PR and the withdrawal batch are computed in a read-only job that
# runs third-party code (release-plz, git-cliff, npm, cargo, the generators),
# and applied, committed and pushed in a job that holds the release token. That
# second job must not take more from the first than the change it exists to
# carry, so before applying the patch it runs this — from its own checkout of
# the commit, before the patch can have changed anything — and refuses:
#
#   - any path outside the producer's allowlist below;
#   - .github/, scripts/, .gitattributes and .gitmodules, whatever the list says;
#   - a symlink or submodule (mode 120000 / 160000), a rename or a copy;
#   - a new executable file (mode 100755, created or changed to), unless the
#     producer's `allow_exec` list names it — neither list names any today;
#   - a path containing a control character;
#   - a patch git cannot read.
#
# The allowlists are what the producers write. `release` is every file a
# `release-pr.sh prepare` run changes (release-plz: the manifests, Cargo.lock
# and the crate changelogs; the bump scripts: each package's manifest, lockfile,
# changelog and version constant). `withdraw` is the spec folders and the four
# generators' output trees, the same trees the drift checks compare.
set -euo pipefail

usage() {
  echo "usage: $0 <release|withdraw> PATCH" >&2
  exit 2
}
[ $# -eq 2 ] || usage
kind="$1"
patch="$2"

case "$kind" in
  release)
    allow_exec=()
    allow=(
      '^Cargo\.(toml|lock)$'
      '^[^/]+/Cargo\.toml$'
      '^[^/]+(/[^/]+)?/CHANGELOG\.md$'
      '^[^/]+/package(-lock)?\.json$'
      '^[^/]+/pubspec\.yaml$'
      '^trust-tasks-go/[^/]+/version\.go$'
      '^trust-tasks-dart/lib/src/runtime/version\.dart$'
    )
    ;;
  withdraw)
    allow_exec=()
    allow=(
      '^specs/'
      '^trust-tasks-rs/(src|tests)/'
      '^trust-tasks-rs/Cargo\.toml$'
      '^trust-tasks-ts/src/'
      '^trust-tasks-ts/component-names\.lock\.json$'
      '^trust-tasks-go/specs/'
      '^trust-tasks-dart/lib/specs/'
    )
    ;;
  *) usage ;;
esac

never='^(\.github|scripts)/|(^|/)\.git(attributes|modules)$'

[ -f "$patch" ] || { echo "::error::$patch does not exist" >&2; exit 1; }
# An empty patch is "nothing to release"; the caller handles it.
[ -s "$patch" ] || exit 0

errors=0
refuse() {
  echo "::error::refusing the patch: $*" >&2
  errors=$((errors + 1))
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# git's own reading of the patch, checked for success: an unreadable patch is
# refused here, not discovered later as an empty file list.
if ! git apply --summary "$patch" >"$tmp/summary" 2>"$tmp/err"; then
  echo "::error::refusing the patch: git cannot read it: $(cat "$tmp/err")" >&2
  exit 1
fi
if ! git apply --numstat -z "$patch" >"$tmp/numstat" 2>"$tmp/err"; then
  echo "::error::refusing the patch: git cannot read it: $(cat "$tmp/err")" >&2
  exit 1
fi

is_allowed_exec() {
  local re
  for re in ${allow_exec[@]+"${allow_exec[@]}"}; do
    [[ "$1" =~ $re ]] && return 0
  done
  return 1
}

# Modes, renames and copies.
while IFS= read -r line; do
  case "$line" in
    *" 120000 "* | *" 120000" | *"=> 120000"*) refuse "symlink: $line" ;;
    *" 160000 "* | *" 160000" | *"=> 160000"*) refuse "submodule: $line" ;;
    " rename "* | " copy "*) refuse "rename or copy: $line" ;;
    " create mode 100755 "*)
      is_allowed_exec "${line# create mode 100755 }" || refuse "new executable file: $line" ;;
    " mode change "*"=> 100755 "*)
      is_allowed_exec "${line#*=> 100755 }" || refuse "made executable: $line" ;;
  esac
done <"$tmp/summary"
[ "$errors" -eq 0 ] || exit 1

# Every path, NUL-separated so no file name can split a record.
count=0
while IFS= read -r -d '' record; do
  path="${record#*$'\t'}"
  path="${path#*$'\t'}"
  count=$((count + 1))
  if [[ "$path" =~ [[:cntrl:]] ]]; then
    refuse "a path contains a control character: $(printf '%q' "$path")"
    continue
  fi
  if [[ "$path" =~ $never ]]; then
    refuse "$path is never part of this change"
    continue
  fi
  ok=0
  for re in "${allow[@]}"; do
    if [[ "$path" =~ $re ]]; then ok=1; break; fi
  done
  [ "$ok" -eq 1 ] || refuse "$path is outside the $kind allowlist"
done <"$tmp/numstat"

if [ "$count" -eq 0 ]; then
  echo "::error::refusing the patch: git found no file in it" >&2
  exit 1
fi
[ "$errors" -eq 0 ] || exit 1
echo "::notice::the patch touches $count file(s), all on the $kind allowlist"
