#!/usr/bin/env bash
# `git push "$@"`, authenticated with $PUSH_TOKEN.
#
# For the release workflows, whose checkouts persist no credentials
# (`persist-credentials: false`): nothing earlier in a job can read a token out
# of `.git/config`, and the token reaches git only here, through the
# environment of this one command — never the config file or the command line.
#
# Only for a checkout of the commit being released: never after applying a
# patch produced elsewhere (release-pr.sh and withdraw.yml push with their
# own copy of these lines for that reason).
set -euo pipefail
# Never trace this script, whatever the caller set: the next lines hold the token.
{ set +x; } 2>/dev/null

: "${PUSH_TOKEN:?PUSH_TOKEN must be set to the token to push with}"

auth=$(printf 'x-access-token:%s' "$PUSH_TOKEN" | base64 | tr -d '\n')
echo "::add-mask::$auth"
GIT_CONFIG_COUNT=1 \
  GIT_CONFIG_KEY_0="http.https://github.com/.extraheader" \
  GIT_CONFIG_VALUE_0="AUTHORIZATION: basic $auth" \
  git push "$@"
