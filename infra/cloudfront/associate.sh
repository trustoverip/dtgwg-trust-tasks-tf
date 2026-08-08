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

# name:eventType. Associated in a single distribution update — each update
# invalidates the ETag, so two sequential updates would need a refetch between
# them.
FUNCTIONS=(
  "trust-tasks-type-uri-negotiation:viewer-request"
)
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

aws cloudfront get-distribution-config --id "$DIST_ID" > "$work/wrapper.json"
etag=$(jq -r '.ETag' "$work/wrapper.json")

# The ARN is stage-independent; CloudFront serves whichever stage is published,
# so each function must be published (LIVE) before association has any effect.
items="[]"
changes=()
for entry in "${FUNCTIONS[@]}"; do
  name="${entry%%:*}"
  event="${entry##*:}"

  arn=$(aws cloudfront describe-function --name "$name" \
          --query 'FunctionSummary.FunctionMetadata.FunctionARN' --output text 2>/dev/null || echo "")
  if [[ -z "$arn" || "$arn" == "None" ]]; then
    echo "error: function $name does not exist — create and publish it first (see README.md)" >&2
    exit 1
  fi
  stage=$(aws cloudfront describe-function --name "$name" --stage LIVE \
            --query 'FunctionSummary.FunctionMetadata.Stage' --output text 2>/dev/null || echo "")
  if [[ "$stage" != "LIVE" ]]; then
    echo "error: $name has no LIVE stage — run publish-function first (see README.md)" >&2
    exit 1
  fi

  current=$(jq -r --arg e "$event" '.DistributionConfig.DefaultCacheBehavior.FunctionAssociations.Items[]?
                   | select(.EventType==$e) | .FunctionARN' "$work/wrapper.json")
  if [[ -n "$current" && "$current" != "$arn" ]]; then
    echo "error: a different $event function is already associated:" >&2
    echo "         $current" >&2
    echo "       refusing to replace it. Resolve by hand." >&2
    exit 1
  fi
  [[ "$current" == "$arn" ]] || changes+=("$event -> $name")

  items=$(jq -c --arg arn "$arn" --arg e "$event" '. + [{FunctionARN:$arn, EventType:$e}]' <<<"$items")
done

if [[ ${#changes[@]} -eq 0 ]]; then
  echo "already associated — nothing to do on $DIST_ID"
  exit 0
fi

# Extract the inner DistributionConfig *and* apply the edit in one pass.
jq --argjson items "$items" '
  .DistributionConfig
  | .DefaultCacheBehavior.FunctionAssociations = {
      Quantity: ($items | length),
      Items: $items
    }
' "$work/wrapper.json" > "$work/config.json"

echo "distribution : $DIST_ID"
echo "if-match     : $etag"
for c in "${changes[@]}"; do echo "change       : associate $c"; done

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
