#!/usr/bin/env bash
# check-npm-tarball.sh TARBALL NAME VERSION
#
# `publish-npm` publishes a tarball that `build-npm` packed, in a job that ran
# third-party code (`npm ci`, the build). Before it goes anywhere, this checks
# the tarball's own package.json, against what the publishing job decided from
# its own checkout:
#
#   - `name` is NAME and `version` is VERSION;
#   - no `publishConfig`, which could send it to another registry or tag;
#   - no `preinstall`, `install` or `postinstall` script, which would run on
#     every consumer's machine. No package here has one.
#
# Reads the tarball with tar and jq only; nothing in it runs.
set -euo pipefail

[ $# -eq 3 ] || { echo "usage: $0 TARBALL NAME VERSION" >&2; exit 2; }
tarball="$1" name="$2" version="$3"

refuse() {
  echo "::error::refusing to publish $tarball: $*" >&2
  exit 1
}

manifest=$(tar -xzOf "$tarball" package/package.json) || refuse "it has no package/package.json"
jq -e 'type == "object"' <<<"$manifest" >/dev/null 2>&1 || refuse "its package.json is not a JSON object"

got_name=$(jq -r '.name' <<<"$manifest")
got_version=$(jq -r '.version' <<<"$manifest")
if [ "$got_name" != "$name" ] || [ "$got_version" != "$version" ]; then
  refuse "it is $got_name@$got_version, not $name@$version"
fi
if jq -e 'has("publishConfig")' <<<"$manifest" >/dev/null; then
  refuse "its package.json has a publishConfig"
fi
if jq -e '(.scripts // {}) | has("preinstall") or has("install") or has("postinstall")' <<<"$manifest" >/dev/null; then
  refuse "its package.json has an install-time script"
fi
echo "::notice::$tarball is $name@$version"
