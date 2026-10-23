---
slug: policy/upsert
version: "0.3"
title: Policy — Upsert
summary: Create or update a Rego policy module on the maintainer. New policies take effect on the next request evaluation; no restart required.
status: draft
targetFrameworkVersion: "0.5.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An upsert writes the policy body wholesale, so two copies applied out of order leave the older text in force. The acceptance window is what lets the maintainer refuse the older one rather than write it.
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

## Changes from 0.2

The response member **`created`** (boolean, required) is renamed
**`wasCreated`** — same type, still required, same meaning: whether this upsert
created the policy rather than updating one.

The registry reserves `created` for a timestamp, so a flag saying whether a
write created the record rather than replacing one is named `wasCreated` on
every create-or-replace task. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains
published.

## Conformance

Producer: populate `name`, `module`. Supply `id` + `expectedVersion` on update. Carry a proof.

Consumer: verify `PolicyAdmin` capability. Parse the Rego with the evaluator (e.g. `regorus`); on parse failure return `regoInvalid` with `details.line/column/message`. Validate `appliesTo` against known context ids. On successful upsert, increment `version`, emit `sync/event/0.1` with kind `policyChanged`, hot-load into the evaluator cache so the next request sees the new rules.

## Security & Privacy

**Sandboxed evaluation.** The Rego evaluator MUST be sandboxed: no I/O, no network, deterministic time. `regorus` provides this by default.

**Replay.** `id` is the idempotency key. A retry with the same id and matching `expectedVersion` is a no-op.

### Data carried

The request carries the policy module itself — `name`, optional `description`, the Rego `module` source, `appliesTo` context ids, `priority`, `enabled` — plus `id` and `expectedVersion` on update. The response echoes the stored `PolicyModule` and `wasCreated`. No member is defined to carry personal data; the Rego source describes the maintainer's security posture rather than any person. The spec imposes no further rule on what a producer puts in `ext`.

### Correlation

`threadId` correlates request and response. The policy `id` is a stable handle across every upsert of the same module, and the required `proof` binds each change to its producer, so successive changes to one policy can be joined to who made them — which is the point of the audit below.

### Retention

**Audit reach.** Every upsert is logged with `{ who, when, policyId, name, diff }`. The maintainer SHOULD retain the previous module source for at least the deletion grace period so a malicious or buggy policy can be diffed and rolled back.

The signed request is evidence of who changed the maintainer's security posture; the spec imposes no further rule on how long it is kept.

### Consent/purpose

The purpose is to change the request-evaluation policy the maintainer applies to every subsequent request. The spec states no further limit on reuse of the data carried.
