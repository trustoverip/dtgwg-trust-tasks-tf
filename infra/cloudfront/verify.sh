#!/usr/bin/env bash
# Verify Type URI content negotiation on the live site (SPEC §6.2).
# Fails until type-uri-negotiation.js is associated with the distribution —
# see README.md. Run after association to confirm it took effect.
set -u
BASE="${1:-https://trusttasks.org}"
fails=0

check() {
  local uri="$1" accept="$2" want_type="$3" want_marker="$4"
  local body ctype
  ctype=$(curl -sS -o /tmp/tt-verify-body -w '%{content_type}' -H "Accept: $accept" "$BASE$uri")
  body=$(head -c 400 /tmp/tt-verify-body)
  if [[ "$ctype" == *"$want_type"* && "$body" == *"$want_marker"* ]]; then
    printf '  ok    %-42s %s\n' "$uri" "$accept"
  else
    printf '  FAIL  %-42s %s\n         got content-type=%s\n' "$uri" "$accept" "$ctype"
    fails=$((fails+1))
  fi
}

echo "Type URI content negotiation — $BASE"
# The framework envelope schema (§7.2 item 1).
check /spec/trust-task/0.2 application/schema+json json '"$id"'
# A task payload schema (§7.2 item 2, §7.3 item 7.5).
check /spec/acl/grant/0.1 application/schema+json json '"$id"'
# Multi-segment slug.
check /spec/did-management/did/delete/0.1 application/schema+json json '"$id"'
# Humans still get prose — the negotiation must not break the site.
check /spec/acl/grant/0.1 'text/html' 'text/html' '<!doctype html'
# SPA routes still render.
check /categories 'text/html' 'text/html' '<!doctype html'

# An unknown Type URI asked for as a schema must be a real 404, not the shell
# under a 200 — see remove-spa-error-mapping.sh. A consumer that cannot tell
# "no such specification" from "here is your schema" is the failure this guards.
check_status() {
  local uri="$1" accept="$2" want="$3"
  local code
  code=$(curl -sS -o /dev/null -w '%{http_code}' -H "Accept: $accept" "$BASE$uri")
  if [[ "$code" == "$want" ]]; then
    printf '  ok    %-42s %s -> %s\n' "$uri" "$accept" "$code"
  else
    printf '  FAIL  %-42s %s -> %s (want %s)\n' "$uri" "$accept" "$code" "$want"
    fails=$((fails+1))
  fi
}
check_status /spec/does-not-exist/0.1 application/schema+json 404
check_status /specs/does/not/exist.schema.json application/schema+json 404
check_status /spec/does-not-exist/0.1 'text/html' 200

if (( fails )); then
  echo "$fails check(s) failed — if the function is not yet associated, that is expected (see README.md)."
  exit 1
fi
echo "all checks passed"
