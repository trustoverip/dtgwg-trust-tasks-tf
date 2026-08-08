# CloudFront content negotiation for Type URIs

`type-uri-negotiation.js` is a CloudFront **viewer-request** function that makes
Type URIs honour `Accept: application/schema+json`, as
[SPEC §6.2](https://trusttasks.org/SPEC#62-content-negotiation) requires.

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
/spec/trust-task/<M.m>  + application/schema+json  ->  /specs/_framework/<M.m>/trust-task.schema.json
/spec/<slug…>/<M.m>     + application/schema+json  ->  /specs/<slug…>/<M.m>/payload.schema.json
anything else                                      ->  unchanged, SPA renders prose
```

Requests without that `Accept` value are untouched, so the human-facing site is
unaffected.

## One-time setup

The function is **inert until associated with the distribution** — deploying the
website does not activate it. Associating it is a manual step, because the
distribution configuration is not managed from this repository.

```sh
# 1. Create (first time only)
aws cloudfront create-function \
  --name trust-tasks-type-uri-negotiation \
  --function-config Comment="Type URI content negotiation (SPEC §6.2)",Runtime=cloudfront-js-2.0 \
  --function-code fileb://type-uri-negotiation.js

# 2. Publish, taking the ETag from the create/describe output
aws cloudfront publish-function \
  --name trust-tasks-type-uri-negotiation --if-match <ETag>

# 3. Associate with the default cache behaviour as a viewer-request function.
#    Fetch the distribution config, add the FunctionAssociation, and update.
aws cloudfront get-distribution-config --id <DISTRIBUTION_ID> > dist.json
#    …add to DefaultCacheBehavior:
#      "FunctionAssociations": { "Quantity": 1, "Items": [
#        { "FunctionARN": "<arn from step 1>", "EventType": "viewer-request" } ] }
aws cloudfront update-distribution --id <DISTRIBUTION_ID> \
  --distribution-config file://dist-config.json --if-match <ETag from get>
```

**Cache behaviour matters.** The origin response varies by `Accept`, so the cache
policy for these paths must include `Accept` in its cache key — otherwise the
first response cached for a path is served to everyone, and either implementers
get HTML or browsers get JSON. Either add `Accept` to the cache policy's header
allowlist or give `/spec/*` its own behaviour.

## Updating it

```sh
aws cloudfront update-function \
  --name trust-tasks-type-uri-negotiation \
  --function-config Comment="Type URI content negotiation (SPEC §6.2)",Runtime=cloudfront-js-2.0 \
  --function-code fileb://type-uri-negotiation.js \
  --if-match <ETag>
aws cloudfront publish-function --name trust-tasks-type-uri-negotiation --if-match <ETag>
```

## Verifying

`./verify.sh` checks the live site. Before association it fails, which is the
expected state until the steps above are run.

## Runtime constraints

CloudFront Functions run a constrained JS engine — ES5.1 idioms only. No
optional chaining, no `String.prototype.includes`, no async. Keep to `indexOf`
and explicit loops when editing.
