# CloudFront content negotiation for registry URIs

`type-uri-negotiation.js` is a CloudFront **viewer-request** function that makes
the registry's published URIs honour `Accept`, as the specification requires:

* **Type URIs** serve their payload schema for `application/schema+json`
  ([SPEC §6.2](https://trusttasks.org/SPEC#62-content-negotiation)).
* **Ceremony definition URIs** serve their definition for `application/json`
  ([SPEC §6.7](https://trusttasks.org/SPEC#67-ceremony-namespace)). Plain
  `application/json` rather than `application/schema+json`, because a definition
  is an *instance* of the ceremony format, not a JSON Schema.

Everything else falls back to the SPA. The file keeps its original name for
continuity; it now negotiates two subtrees rather than one.

## Why this exists

Three normative requirements depend on it, and none of them worked before it:

| Requirement | What it says |
|---|---|
| §6.2 | "A server hosting a *Type URI* **MUST** support HTTP content negotiation" |
| §7.2 items 1–2 | Both mandatory consumer validation steps fetch their schema by content-negotiating a Type URI |
| §7.3 item 7.5 | Every specification's payload schema "is served at its *Type URI* under content negotiation" |

Every Type URI returned the SPA shell as `text/html` regardless of `Accept`. The
schemas were deployed the whole time, just at `/specs/<slug>/<version>/payload.schema.json`
— a different URL from the one the specification tells implementers to use.

## What it does

```
/spec/trust-task/<M.m>   + application/schema+json ->  /specs/_framework/<M.m>/trust-task.schema.json
/spec/<slug…>/<M.m>      + application/schema+json ->  /specs/<slug…>/<M.m>/payload.schema.json
/ceremony/<slug…>/<M.m>  + application/json        ->  /ceremonies/<slug…>/<M.m>/ceremony.json
/assets/…, /specs/…, /bindings/…, /ceremonies/…,
  root files                                       ->  unchanged (a missing one 404s honestly)
anything else                                      ->  /index.html (SPA renders the route)
```

The `/spec/` and `/ceremony/` subtrees are **structurally disjoint** (SPEC §6.7),
and the function keeps them so: a Type URI requested as `application/json` does
not fall into the ceremony branch, and a definition URI requested as
`application/schema+json` does not fall into the schema branch. The singular and
plural forms of each (`/spec/` vs `/specs/`, `/ceremony/` vs `/ceremonies/`)
share a long common prefix and diverge only at the character after it — which is
why the asset-tree guard is a prefix test on the plural and the negotiation is a
prefix test on the singular. Reversing either would loop, and a substring test
rather than a `indexOf(…) === 0` prefix test would match both.

The second job used to be a distribution-level `404 -> /index.html as 200`
custom error response. That is the conventional SPA catch-all and fine for
browsers, but it meant an unknown or mistyped Type URI returned the site shell
under a **success** status — so a consumer following §7.2 items 1–2 ("fetch the
schema and validate against it") quietly validated against nothing.

It could not be narrowed in place. `CustomErrorResponses` is distribution-wide
with no per-cache-behaviour override, and it cannot be corrected on the way out
either: **viewer-response functions do not run for responses CloudFront
generates itself**, and a custom error page is exactly that. (Confirmed
empirically — a viewer-response function fired for `/` and never for
`/spec/does-not-exist/0.1`.) So the mapping is removed and the fallback happens
in the viewer-request function, where the rule is ours to scope.

The rule is inverted from the usual "no file extension means an SPA route",
which fails here because `/spec/acl/grant/0.1` ends in what looks like a `.1`
extension. Instead everything is an SPA route *except* the real asset trees and
root files — precisely what should 404 when absent. Nothing can be missed,
because the SPA is the default.

⚠️ `ROOT_FILES` in the function must track the root of `website/`. A new root
file added without updating it is served the shell instead of itself.

⚠️ `ASSET_PREFIXES` must track the published trees. A tree synced into
`website/` without being listed there is swallowed by the SPA fallback and
served as `text/html` — which is exactly how `/ceremonies/` behaved between the
tree being published and this function learning about it.

## One-time setup

The function is **inert until associated with a distribution** — deploying the
website does not activate it, and the distribution configuration is not managed
from this repository.

### Create and publish the function

```sh
aws cloudfront create-function \
  --name trust-tasks-type-uri-negotiation \
  --function-config Comment="Type URI + ceremony definition negotiation with SPA fallback (SPEC 6.2 / 6.7)",Runtime=cloudfront-js-2.0 \
  --function-code fileb://type-uri-negotiation.js

# The ETag comes from the create response (or `describe-function`).
aws cloudfront publish-function \
  --name trust-tasks-type-uri-negotiation \
  --if-match "$(aws cloudfront describe-function --name trust-tasks-type-uri-negotiation \
                  --query 'ETag' --output text)"
```

### Associate it with the distribution

```sh
./associate.sh <DISTRIBUTION_ID>          # dry run — prints the change, touches nothing
./associate.sh <DISTRIBUTION_ID> --apply  # performs it
```

Use the script rather than doing this by hand. `get-distribution-config` returns
a **wrapper**:

```json
{ "ETag": "E13V…", "DistributionConfig": { … } }
```

but `update-distribution --distribution-config` expects only the **inner**
object. Passing the wrapper fails with a misleading pile of errors —

```
Missing required parameter in DistributionConfig: "CallerReference"
Missing required parameter in DistributionConfig: "Origins"
Unknown parameter in DistributionConfig: "ETag", must be one of: …
```

— which reads as though the config is malformed rather than double-wrapped. The
script extracts `.DistributionConfig` and applies the edit in one `jq` pass, so
the two cannot drift. It is idempotent, and refuses to act if a *different*
viewer-request function is already associated.

The `--if-match` value must be the `ETag` from that same
`get-distribution-config` response. It is not a fixed constant, and it changes
on every distribution update.

### Remove the SPA error mapping

⚠️ **Order matters.** Update and publish the function *first*, then remove the
mapping. The other way round 404s every client-side route until the function
catches up. `remove-spa-error-mapping.sh` refuses to run unless the deployed
LIVE function already contains the fallback, so the order is enforced rather
than remembered.

```sh
./remove-spa-error-mapping.sh <DISTRIBUTION_ID>          # dry run
./remove-spa-error-mapping.sh <DISTRIBUTION_ID> --apply
```

The `403` mapping is left in place. It is harmless *provided* the bucket policy
grants `s3:ListBucket` — see below, because without that this whole change is
undone.

### The bucket policy is part of this

The distribution's origin must be able to return a real **404**, and with
CloudFront + OAC that depends on the bucket policy:

```json
{
  "Sid": "AllowCloudFrontListBucketForRealNotFound",
  "Effect": "Allow",
  "Principal": { "Service": "cloudfront.amazonaws.com" },
  "Action": "s3:ListBucket",
  "Resource": "arn:aws:s3:::<bucket>",
  "Condition": { "ArnLike": { "AWS:SourceArn": "<distribution ARN>" } }
}
```

Without `s3:ListBucket`, S3 cannot distinguish a missing key from a forbidden
one and answers **403 AccessDenied**. The remaining `403 -> /index.html as 200`
mapping then turns that back into the site shell under a success status —
reinstating precisely the bug the 404 removal was meant to fix.

It sounds broader than it is. CloudFront never issues a list operation on a
viewer's behalf and the distribution offers no way to trigger one; the grant
only changes which error S3 returns for a missing key. Scope it to the same
principal and `SourceArn` condition as the existing `s3:GetObject` statement.

⚠️ **Check this with `aws s3api get-bucket-policy`, not by calling
`head-object` yourself.** An IAM user with broader permissions gets a clean 404
and tells you nothing about what the CloudFront principal sees. That false
signal is why the 403 mapping was originally left in place with a comment
asserting the opposite.

### Cache keys need no change

A viewer-request function runs **before** the cache lookup, and the cache key is
computed from the URI it produces. A request for
`/spec/acl/grant/0.1` with `Accept: application/schema+json` is rewritten to
`/specs/acl/grant/0.1/payload.schema.json` before the lookup, so it occupies a
different cache entry from the un-rewritten HTML request for the same Type URI.
The two representations cannot collide.

So the `Managed-CachingOptimized` policy (`HeaderBehavior: none`) is fine as-is —
adding `Accept` to the cache key is unnecessary, and would only fragment the
cache.

## Updating the function

```sh
etag=$(aws cloudfront describe-function --name trust-tasks-type-uri-negotiation \
         --query 'ETag' --output text)
aws cloudfront update-function \
  --name trust-tasks-type-uri-negotiation \
  --function-config Comment="Type URI + ceremony definition negotiation with SPA fallback (SPEC 6.2 / 6.7)",Runtime=cloudfront-js-2.0 \
  --function-code fileb://type-uri-negotiation.js \
  --if-match "$etag"

# update-function returns a fresh ETag; publish with that one.
aws cloudfront publish-function --name trust-tasks-type-uri-negotiation \
  --if-match "$(aws cloudfront describe-function --name trust-tasks-type-uri-negotiation \
                  --query 'ETag' --output text)"
```

Re-association is not needed — the distribution references the function by ARN
and always serves the published (LIVE) stage.

⚠️ **No commas in `Comment`.** `--function-config` uses the shorthand
`key=value,key=value` syntax, so a comma inside the comment is parsed as a field
separator and the call fails with `Invalid type for parameter
FunctionConfig.Comment … type: <class 'list'>`, which does not obviously point at
the comment. Keep the value comma-free, or pass the config as JSON.

⚠️ **Carry the comment forward.** `update-function` replaces the whole config, so
omitting `Comment` — or pasting an older one from these examples — silently
reverts it. Check what is deployed first with `describe-function`.

### Before publishing: check the deployed function is what you think

The distribution is not managed from this repository, so the LIVE function can
differ from `main`. Diff before you publish, or you ship someone else's
undeployed change along with your own:

```sh
aws cloudfront get-function --name trust-tasks-type-uri-negotiation \
  --stage LIVE /tmp/live-fn.js
diff /tmp/live-fn.js type-uri-negotiation.js
```

### Test in the real runtime, not just in Node

`npm run test:infra` exercises the routing logic under Node. CloudFront runs a
different, constrained engine, so a passing test suite does not prove the
function works where it will actually run. `test-function` executes the
DEVELOPMENT stage against a synthetic event and returns the rewritten request:

```sh
cat > /tmp/ev.json <<'EOF'
{"version":"1.0","context":{"eventType":"viewer-request"},"viewer":{"ip":"1.2.3.4"},
 "request":{"method":"GET","uri":"/ceremony/vtc/member-onboarding/0.1","querystring":{},
            "headers":{"accept":{"value":"application/json"}},"cookies":{}}}
EOF

aws cloudfront test-function --name trust-tasks-type-uri-negotiation \
  --stage DEVELOPMENT --if-match "$etag" --event-object fileb:///tmp/ev.json \
  --query 'TestResult.FunctionOutput' --output text
```

Do this between `update-function` and `publish-function` — that is the window in
which the new code exists but is not yet serving traffic.

### ⚠️ The rewrite target must exist before the function goes live

A negotiation rule points at an object on the origin. Publish the function before
that object is deployed and the URI 404s for every client that asks for it.

So when adding a subtree: **merge and deploy the website first, then publish the
function.** Verify the object is really there with `aws s3 ls` rather than
through CloudFront — until `ASSET_PREFIXES` knows about the tree, CloudFront
serves the SPA shell for it and tells you nothing about whether the object
exists.

This is the mirror image of the ordering rule for the SPA error mapping below:
there the function must lead, here it must follow. The principle underneath both
is the same — never let a rule go live ahead of the thing it depends on.

## Verifying

`./verify.sh` checks the live site — Type URI negotiation, ceremony definition
negotiation, that humans still get prose, and that an unknown URI in either
subtree is a real 404 rather than the shell under a 200. Before association it
fails, which is the expected state until the steps above are run.

## Runtime constraints

CloudFront Functions run a constrained JS engine — ES5.1 idioms only. No
optional chaining, no `String.prototype.includes`, no async. Keep to `indexOf`
and explicit loops when editing.
