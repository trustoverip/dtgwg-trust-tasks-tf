#!/usr/bin/env bash
# Verify Type URI content negotiation (SPEC §6.2) and the response headers on
# the live site.
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
# The three-part framework Type URI (SPEC §5.1.1), and the .0 alias it shares.
check /spec/trust-task/0.5.0 application/schema+json json '"$id"'
check /spec/trust-task/0.6.0 application/schema+json json '"$id"'
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

# Ceremony definition URIs (SPEC §6.7). The subtree is structurally disjoint
# from /spec/, so these also guard against one resolving into the other's tree.
echo
echo "Ceremony definition negotiation — $BASE"
# A definition served as JSON at the URI a step's ceremony.definition names.
check /ceremony/vtc/member-onboarding/0.1 application/json json '"slug"'
# A person asking for the same URI gets the page, not the document.
check /ceremony/vtc/member-onboarding/0.1 'text/html' 'text/html' '<!doctype html'
# The raw asset tree is served directly and must not be rewritten back into itself.
check /ceremonies/vtc/member-onboarding/0.1/ceremony.json application/json json '"slug"'
# An unknown definition must 404 honestly, for the same reason an unknown Type
# URI must: a consumer cannot distinguish "no such ceremony" from a definition
# it failed to parse if both arrive as the site shell under a 200.
check_status /ceremony/does-not-exist/9.9 application/json 404
# A Type URI asked for as plain JSON is not a definition — the two subtrees must
# not leak into each other. It names nothing negotiable, so it is a 404 (#1152);
# it used to be the SPA shell under a 200.
check_status /spec/acl/grant/0.1 application/json 404
# Malformed Type URIs asked for as a schema: outside the slug grammar, and with
# no version. A browser still gets the shell for both.
check_status /spec/Nope/0.1 application/schema+json 404
check_status /spec/acl/grant application/schema+json 404
check_status /spec/acl/grant 'text/html' 200
# Root files the build writes (#1150); without a ROOT_FILES entry each is the shell.
check /robots.txt '*/*' 'text/plain' 'Sitemap:'
check /sitemap.xml '*/*' 'xml' '<urlset'

# Response headers (response-headers-policy.json, applied by
# apply-response-headers.sh). Checked on the page and on a negotiated schema,
# since one policy covers every cache behaviour.
echo
echo "Response headers — $BASE"
check_header() {
  local uri="$1" accept="$2" header="$3" want="$4"
  local value
  # Sent as a cross-origin request: CloudFront adds a response headers policy's
  # CORS headers only when the request carries an Origin, as a browser's fetch
  # from another site does. Without it Access-Control-Allow-Origin is (rightly)
  # absent and the check would fail on a correctly configured distribution.
  value=$(curl -sS -o /dev/null -D - -H "Accept: $accept" -H "Origin: https://example.org" "$BASE$uri" \
            | tr -d '\r' | awk -v h="$(tr '[:upper:]' '[:lower:]' <<<"$header")" \
                'BEGIN{FS=": "} tolower($1)==h {sub(/^[^:]*: /, ""); print; exit}')
  if [[ "$value" == *"$want"* ]]; then
    printf '  ok    %-42s %s\n' "$uri" "$header"
  else
    printf '  FAIL  %-42s %s (got: %s; want it to contain: %s)\n' "$uri" "$header" "${value:-<absent>}" "$want"
    fails=$((fails+1))
  fi
}
for uri in / /spec/acl/grant/0.1; do
  check_header "$uri" 'text/html' Strict-Transport-Security 'max-age='
  check_header "$uri" 'text/html' X-Content-Type-Options nosniff
  check_header "$uri" 'text/html' X-Frame-Options DENY
  check_header "$uri" 'text/html' Referrer-Policy strict-origin-when-cross-origin
  check_header "$uri" 'text/html' Content-Security-Policy "frame-ancestors 'none'"
  check_header "$uri" 'text/html' Permissions-Policy 'camera=()'
done
check_header /spec/acl/grant/0.1 application/schema+json Access-Control-Allow-Origin '*'
check_header /spec/acl/grant/0.1 application/schema+json Vary Accept
check_header /spec/acl/grant/0.1 application/schema+json X-Content-Type-Options nosniff

if (( fails )); then
  echo "$fails check(s) failed — if the function is not yet associated, that is expected (see README.md)."
  exit 1
fi
echo "all checks passed"
