---
slug: policy/delete
version: "0.1"
title: Policy — Delete
summary: Delete a Rego policy module; takes effect on the next request evaluation.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - policy
  - delete
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
  - role: vault maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Removing a policy changes the maintainer's security posture; producer identity MUST be verifiable for audit.
errorCodes:
  - code: policy/delete:not_found
    meaning: No policy with this id.
    retryable: false
  - code: policy/delete:permission_denied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
  - code: policy/delete:version_conflict
    meaning: "`expectedVersion` mismatch."
    retryable: true
  - code: policy/delete:would_orphan_contexts
    meaning: Deleting this policy would leave one or more contexts with no applicable policy. The maintainer's policy on this is configurable — if deny-by-default is in place, this is benign; if the policy was the only `allow` for the context, you'd be locking yourself out. Override by setting an `ext` flag.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        orphanedContexts: { type: "array", items: { "type": "string" } }
---

## Abstract

Removes a policy module. The maintainer SHOULD warn (via `would_orphan_contexts`) if the deletion would leave any context with no applicable policy that allows access; the user can override via `ext` flag if they understand the implications.

## Conformance

Producer: populate `id`; SHOULD populate `expectedVersion`. Consumer: verify `PolicyAdmin`, check orphan condition, emit `sync/event/0.1` with kind `policy.changed`, hot-unload from evaluator cache.

## Security & Privacy

**Orphan check.** Same intent as the upsert audit retention: a misclick should not silently lock the user out. The deny-by-default fallback is the safety net; the orphan warning is the first line.

**Audit.** Logged with `{ who, when, policyId, reason? }`. The deleted policy's source SHOULD be retained in audit-only storage for rollback.
