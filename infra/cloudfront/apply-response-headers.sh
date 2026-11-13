#!/usr/bin/env bash
# Create or update the site's CloudFront response headers policy from
# response-headers-policy.json, and attach it to every cache behaviour of a
# distribution. See README.md, "Response headers".
#
#   ./apply-response-headers.sh <DISTRIBUTION_ID> [--apply]
#
# Without --apply it prints what it would change and exits, touching nothing.
#
# Idempotent: re-running with an unchanged policy file updates the policy to
# the same content and leaves the distribution alone. It refuses to replace a
# DIFFERENT response headers policy already attached to a cache behaviour —
# a distribution-level change nobody reviewed here should be resolved by hand,
# not overwritten.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
POLICY_FILE="$here/response-headers-policy.json"
DIST_ID="${1:-}"
APPLY="${2:-}"

if [[ -z "$DIST_ID" ]]; then
  echo "usage: $0 <DISTRIBUTION_ID> [--apply]" >&2
  exit 64
fi

for tool in aws jq; do
  command -v "$tool" >/dev/null || { echo "error: $tool is required" >&2; exit 69; }
done

name=$(jq -r '.Name' "$POLICY_FILE")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# --- 1. The policy itself -------------------------------------------------
policy_id=$(aws cloudfront list-response-headers-policies --type custom \
  --query "ResponseHeadersPolicyList.Items[?ResponseHeadersPolicy.ResponseHeadersPolicyConfig.Name=='$name'].ResponseHeadersPolicy.Id | [0]" \
  --output text)
[[ "$policy_id" == "None" ]] && policy_id=""

if [[ -z "$policy_id" ]]; then
  echo "policy       : $name does not exist — would create it from $(basename "$POLICY_FILE")"
else
  aws cloudfront get-response-headers-policy-config --id "$policy_id" > "$work/live-policy.json"
  if diff <(jq -S '.ResponseHeadersPolicyConfig' "$work/live-policy.json") <(jq -S . "$POLICY_FILE") > "$work/policy.diff"; then
    echo "policy       : $name ($policy_id) matches $(basename "$POLICY_FILE")"
  else
    echo "policy       : $name ($policy_id) differs from $(basename "$POLICY_FILE") — would update it:"
    sed 's/^/               /' "$work/policy.diff"
  fi
fi

# --- 2. The distribution's cache behaviours -------------------------------
aws cloudfront get-distribution-config --id "$DIST_ID" > "$work/wrapper.json"
etag=$(jq -r '.ETag' "$work/wrapper.json")

# Every behaviour's current policy: the default first, then each path pattern.
jq -r '
  ["default", (.DistributionConfig.DefaultCacheBehavior.ResponseHeadersPolicyId // "")],
  (.DistributionConfig.CacheBehaviors.Items[]? | [.PathPattern, (.ResponseHeadersPolicyId // "")])
  | @tsv' "$work/wrapper.json" > "$work/behaviours.tsv"

needs_attach=0
while IFS=$'\t' read -r behaviour current; do
  if [[ -z "$current" ]]; then
    echo "behaviour    : $behaviour — no response headers policy; would attach $name"
    needs_attach=1
  elif [[ -n "$policy_id" && "$current" == "$policy_id" ]]; then
    echo "behaviour    : $behaviour — already uses $name"
  else
    echo "error: cache behaviour '$behaviour' already uses a different response headers policy ($current)." >&2
    echo "       Refusing to replace it. Fold its headers into $(basename "$POLICY_FILE") and detach it by hand." >&2
    exit 1
  fi
done < "$work/behaviours.tsv"

if [[ "$APPLY" != "--apply" ]]; then
  echo
  echo "dry run — nothing changed. Re-run with --apply."
  exit 0
fi

if [[ -z "$policy_id" ]]; then
  policy_id=$(aws cloudfront create-response-headers-policy \
    --response-headers-policy-config "file://$POLICY_FILE" \
    --query 'ResponseHeadersPolicy.Id' --output text)
  echo "created      : $name ($policy_id)"
else
  aws cloudfront update-response-headers-policy --id "$policy_id" \
    --response-headers-policy-config "file://$POLICY_FILE" \
    --if-match "$(jq -r '.ETag' "$work/live-policy.json")" \
    --query 'ResponseHeadersPolicy.Id' --output text >/dev/null
  echo "updated      : $name ($policy_id)"
fi

if (( needs_attach )); then
  # Extract the inner DistributionConfig and set the policy on every behaviour
  # in one pass — see associate.sh for why the wrapper must not be passed back.
  jq --arg id "$policy_id" '
    .DistributionConfig
    | .DefaultCacheBehavior.ResponseHeadersPolicyId = $id
    | if .CacheBehaviors.Items then .CacheBehaviors.Items |= map(.ResponseHeadersPolicyId = $id) else . end
  ' "$work/wrapper.json" > "$work/config.json"
  aws cloudfront update-distribution \
    --id "$DIST_ID" \
    --distribution-config "file://$work/config.json" \
    --if-match "$etag" \
    --query 'Distribution.Status' --output text
  echo "attached     : $name to every cache behaviour of $DIST_ID"
fi

echo "submitted. CloudFront takes a few minutes to reach Deployed; then run ./verify.sh"
