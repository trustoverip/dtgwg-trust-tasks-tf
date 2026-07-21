---
slug: policy/get
version: "0.1"
title: Policy — Get
summary: Fetch a single Rego policy module by id, including its source. The read-one companion to policy/list.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - rego
  - get
  - read
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
  requirement: RECOMMENDED
  rationale: Read-only fetch of a single policy. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Read-only fetch of one policy module; persists nothing."
subjectPath: /id
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: policy/get:permissionDenied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
  - code: policy/get:notFound
    meaning: No policy module with the supplied `id` exists.
    retryable: false
---

## Abstract

The **Policy — Get** Trust Task returns one Rego policy module by its `id`, including the full `module` source. It is the read-one companion to [`policy/list`](../../list/0.2/), which has no `id` filter and so cannot address a single policy: a consumer that knows a policy id — from an earlier `policy/upsert` response, a `policy/list` page, or an audit record — fetches it here rather than paging the whole list and filtering client-side. The distinction matters for authorization and failure semantics: `get` returns `notFound` for an unknown id, where a filtered list would return an empty page.

## Conformance

Producer: supply `id`.

Consumer: verify `PolicyAdmin` capability (the same gate `policy/list` and `policy/upsert` apply). Resolve the policy by `id`; if none exists, return `notFound`. On success, return the full `PolicyModule` — the same shape `policy/list` returns per item and `policy/upsert` returns in `policy` — including `module`, `version`, and `appliesTo`.

## Security & Privacy

**Source disclosure.** Rego source is not a secret, but it describes the maintainer's security posture and — as with `policy/list` — SHOULD be visible only to admin-class consumers. The `PolicyAdmin` gate is what enforces that; `get` does not relax it.

**Enumeration.** Because an unknown `id` returns `notFound` while a real id returns the policy, `get` is an existence oracle for policy ids. Policy ids are maintainer-minted opaque identifiers, not guessable subject material, so this is acceptable; do not switch the identifier to anything caller-supplied or enumerable without revisiting this.

**Auditing.** Recommended at sampled rate, matching `policy/list`.
