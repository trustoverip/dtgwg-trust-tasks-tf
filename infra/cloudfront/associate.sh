#!/usr/bin/env bash
# Associate the Type URI negotiation function with a CloudFront distribution.
#
# The manual version of this is easy to get wrong in one specific way:
# `get-distribution-config` returns a WRAPPER —
#
#     { "ETag": "...", "DistributionConfig": { ... } }
#
# — while `update-distribution --distribution-config` expects only the inner
# object. Passing the wrapper produces a confusing pile of "Missing required
# parameter in DistributionConfig" errors alongside "Unknown parameter ETag",
# which reads like the config is malformed rather than double-wrapped. The jq
# below extracts and edits in one pass, so the two cannot drift apart.
#
# Idempotent: re-running when the function is already associated is a no-op.
#
#   ./associate.sh <DISTRIBUTION_ID> [--apply]
#
# Without --apply it prints the change and exits, touching nothing.

set -euo pipefail

FUNCTION_NAME="trust-tasks-type-uri-negotiation"
DIST_ID="${1:-}"
APPLY="${2:-}"

if [[ -z "$DIST_ID" ]]; then
  echo "usage: $0 <DISTRIBUTION_ID> [--apply]" >&2
  exit 64
fi

for tool in aws jq; do
  command -v "$tool" >/dev/null || { echo "error: $tool is required" >&2; exit 69; }
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The ARN is stage-independent; CloudFront serves whichever stage is published,
# so the function must be published (LIVE) before this has any effect.
arn=$(aws cloudfront describe-function --name "$FUNCTION_NAME" \
        --query 'FunctionSummary.FunctionMetadata.FunctionARN' --output text)
stage=$(aws cloudfront describe-function --name "$FUNCTION_NAME" --stage LIVE \
        --query 'FunctionSummary.FunctionMetadata.Stage' --output text 2>/dev/null || echo "")
if [[ "$stage" != "LIVE" ]]; then
  echo "error: $FUNCTION_NAME has no LIVE stage — run publish-function first (see README.md)" >&2
  exit 1
fi

aws cloudfront get-distribution-config --id "$DIST_ID" > "$work/wrapper.json"
etag=$(jq -r '.ETag' "$work/wrapper.json")

current=$(jq -r '.DistributionConfig.DefaultCacheBehavior.FunctionAssociations.Items[]?
                 | select(.EventType=="viewer-request") | .FunctionARN' "$work/wrapper.json")
if [[ "$current" == "$arn" ]]; then
  echo "already associated — $FUNCTION_NAME is the viewer-request function on $DIST_ID"
  exit 0
fi
if [[ -n "$current" ]]; then
  echo "error: a different viewer-request function is already associated:" >&2
  echo "         $current" >&2
  echo "       refusing to replace it. Resolve by hand." >&2
  exit 1
fi

# Extract the inner DistributionConfig *and* apply the edit in one pass.
jq --arg arn "$arn" '
  .DistributionConfig
  | .DefaultCacheBehavior.FunctionAssociations = {
      Quantity: 1,
      Items: [ { FunctionARN: $arn, EventType: "viewer-request" } ]
    }
' "$work/wrapper.json" > "$work/config.json"

echo "distribution : $DIST_ID"
echo "function     : $arn"
echo "if-match     : $etag"
echo "change       : DefaultCacheBehavior.FunctionAssociations 0 -> 1 (viewer-request)"

if [[ "$APPLY" != "--apply" ]]; then
  echo
  echo "dry run — nothing changed. Re-run with --apply to associate."
  exit 0
fi

aws cloudfront update-distribution \
  --id "$DIST_ID" \
  --distribution-config "file://$work/config.json" \
  --if-match "$etag" \
  --query 'Distribution.Status' --output text

echo "submitted. CloudFront takes a few minutes to reach Deployed; then run ./verify.sh"
