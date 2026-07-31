---
slug: policy/upsert
version: "0.2"
wireCompatibleWith: "0.1"
title: Policy — Upsert
summary: Create or update a Rego policy module on the maintainer. New policies take effect on the next request evaluation; no restart required.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - rego
  - upsert
  - hot-reload
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Policy changes alter the maintainer's security posture for every subsequent request. The producer's identity MUST be verifiable for audit and to prevent stealth modifications.
sideEffects:
  level: mutating
  rationale: "Creates or updates a Rego policy module; recoverable by upserting again."
consequences:
  - "Changes request-evaluation policy for every subsequent request, effective immediately."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: policy/upsert:permissionDenied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
  - code: policy/upsert:notFound
    meaning: An `id` was supplied for update but no policy with that id exists.
    retryable: false
  - code: policy/upsert:versionConflict
    meaning: "`expectedVersion` does not match."
    retryable: true
  - code: policy/upsert:regoInvalid
    meaning: The supplied `module` failed Rego parsing or static analysis.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        line: { type: "integer", minimum: 1 }
        column: { type: "integer", minimum: 1 }
        message: { type: "string" }
  - code: policy/upsert:contextNotFound
    meaning: An entry in `appliesTo` references a context that does not exist.
    retryable: false
---

## Abstract

The **Policy — Upsert** Trust Task creates or updates a Rego policy module. The maintainer parses and validates the source before persisting; the new policy is hot-loaded and takes effect on the next request evaluation.

## Conformance

Producer: populate `name`, `module`. Supply `id` + `expectedVersion` on update. Carry a proof.

Consumer: verify `PolicyAdmin` capability. Parse the Rego with the evaluator (e.g. `regorus`); on parse failure return `regoInvalid` with `details.line/column/message`. Validate `appliesTo` against known context ids. On successful upsert, increment `version`, emit `sync/event/0.1` with kind `policyChanged`, hot-load into the evaluator cache so the next request sees the new rules.

## Security & Privacy

**Sandboxed evaluation.** The Rego evaluator MUST be sandboxed: no I/O, no network, deterministic time. `regorus` provides this by default.

**Audit reach.** Every upsert is logged with `{ who, when, policyId, name, diff }`. The maintainer SHOULD retain the previous module source for at least the deletion grace period so a malicious or buggy policy can be diffed and rolled back.

**Replay.** `id` is the idempotency key. A retry with the same id and matching `expectedVersion` is a no-op.
