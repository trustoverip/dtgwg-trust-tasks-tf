#!/usr/bin/env bash
# Remove the `404 -> /index.html as 200` custom error response.
#
# That mapping is the conventional single-page-app catch-all and is fine for
# browsers, but it means an unknown or mistyped Type URI returns the site shell
# under a **success** status. SPEC §7.2 items 1–2 require a consumer to fetch a
# schema and validate against it, so the failure surfaces as a JSON parse error
# at best and as "validated against nothing" at worst.
#
# It cannot be narrowed in place: `CustomErrorResponses` is distribution-wide
# with no per-cache-behaviour override, and it cannot be corrected on the way out
# either, because viewer-response functions do not run for responses CloudFront
# generates itself — which is exactly what a custom error page is. So the
# mapping goes, and type-uri-negotiation.js does the SPA fallback instead.
#
# ⚠ ORDER MATTERS. The viewer-request function must already carry the SPA
# fallback and be published LIVE before this runs. Removing the mapping first
# would 404 every client-side route until the function catches up. This script
# refuses to proceed unless the deployed function contains the fallback.
#
#   ./remove-spa-error-mapping.sh <DISTRIBUTION_ID> [--apply]
#
# The 403 mapping is left in place, but only works because the bucket policy
# grants the CloudFront principal `s3:ListBucket`. Without it S3 cannot tell a
# missing key from a forbidden one and answers 403, which the 403 mapping then
# turns back into the shell under a 200 — reinstating the exact bug this
# removes. See README.md "The bucket policy is part of this". Verified with
# `aws s3api get-bucket-policy`, not with your own credentials: an IAM user with
# broader permissions gets a 404 from `head-object` and tells you nothing about
# what CloudFront sees.

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

# Guard: the published function must already handle the SPA fallback.
if ! aws cloudfront get-function --name "$FUNCTION_NAME" --stage LIVE "$work/live.js" >/dev/null 2>&1; then
  echo "error: $FUNCTION_NAME has no LIVE stage — publish it before removing the mapping" >&2
  exit 1
fi
if ! grep -q "ASSET_PREFIXES" "$work/live.js"; then
  echo "error: the LIVE $FUNCTION_NAME does not contain the SPA fallback." >&2
  echo "       Removing the mapping now would 404 every client-side route." >&2
  echo "       Update and publish the function first (see README.md)." >&2
  exit 1
fi

aws cloudfront get-distribution-config --id "$DIST_ID" > "$work/wrapper.json"
etag=$(jq -r '.ETag' "$work/wrapper.json")

before=$(jq -r '[.DistributionConfig.CustomErrorResponses.Items[]? | select(.ErrorCode==404)] | length' "$work/wrapper.json")
if [[ "$before" == "0" ]]; then
  echo "already removed — no 404 custom error response on $DIST_ID"
  exit 0
fi

jq '.DistributionConfig
    | .CustomErrorResponses.Items =
        [ .CustomErrorResponses.Items[] | select(.ErrorCode != 404) ]
    | .CustomErrorResponses.Quantity = (.CustomErrorResponses.Items | length)
   ' "$work/wrapper.json" > "$work/config.json"

echo "distribution : $DIST_ID"
echo "if-match     : $etag"
echo "change       : drop CustomErrorResponses 404 -> /index.html (200)"
echo "remaining    :"
jq -r '.CustomErrorResponses.Items[]? | "                 \(.ErrorCode) -> \(.ResponsePagePath) as \(.ResponseCode)"' "$work/config.json"

if [[ "$APPLY" != "--apply" ]]; then
  echo
  echo "dry run — nothing changed. Re-run with --apply."
  exit 0
fi

aws cloudfront update-distribution \
  --id "$DIST_ID" \
  --distribution-config "file://$work/config.json" \
  --if-match "$etag" \
  --query 'Distribution.Status' --output text

echo "submitted. Wait for Deployed, then run ./verify.sh"
