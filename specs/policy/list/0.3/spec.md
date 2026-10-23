---
slug: policy/list
version: "0.3"
title: Policy — List
summary: List Rego policy modules registered on the maintainer, optionally filtered by context or enablement status.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - policy
  - rego
  - opa
  - authorization
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
  rationale: Read-only inventory.
sideEffects:
  level: none
  rationale: "Read-only listing of registered policy modules."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **Policy — List** Trust Task returns the registered Rego policy modules. Drives the policy-editor UI in the browser plugin.

## Changes from 0.2

Paging moves to the registry-wide pagination convention — request `cursor` + `limit`, response `nextCursor` absent on the last page — so every list task pages the same way:

* Request **`pageSize`** → **`limit`**. Same type and bounds (integer, 1–1000).
* Response **`cursor`** → **`nextCursor`**. Same opaque string, sent back as the next request's `cursor`.
* Response **`truncated`** is **removed**, and `policies` is now the only required response member. The presence of `nextCursor` says more matching policies remain; its absence marks the last page.

The request's `cursor` member keeps its name and meaning. Nothing else in the payload moves. 0.2's `wireCompatibleWith: "0.1"` declaration is not carried forward, since this version is not wire-identical to `0.1`.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

Producer: optional filters. Consumer: verify `PolicyAdmin` (or read-only `policy-read` if such a capability is defined later). Return policies in `priority` descending order, then `updatedAt` descending. Honor `limit`; return a `nextCursor` iff more matching policies remain, and refuse a `cursor` it did not issue. A producer treats `nextCursor` as opaque and sends it back verbatim as `cursor`.

## Security & Privacy

### Data carried

The request carries optional filters (`contextId`, `enabledOnly`) and paging members. The response carries each matching `PolicyModule` in full: its `id`, `name`, `description`, Rego `module` source, `appliesTo` context ids, `priority`, `enabled` flag, `version` and timestamps. No member is defined to carry personal data; the spec imposes no further rule on what a producer puts in `ext`.

**Source disclosure.** Rego sources are NOT secrets, but they describe the maintainer's security posture and SHOULD be visible only to admin-class consumers.

### Correlation

`threadId` correlates request and response, and `nextCursor` links the pages of one listing. Policy `id`s and `version`s are stable across calls, so a consumer that lists repeatedly can track when each policy changed. The spec imposes no further rule.

### Retention

The maintainer retains nothing on account of this task; it reads modules it already holds.

**Auditing list calls.** Recommended at sampled rate.

The spec imposes no rule on how long a consumer keeps the listing it receives.

### Consent/purpose

The purpose is to drive the policy-editor UI in the browser plugin: letting an administrator see which policy modules are registered and in what order they evaluate. The spec states no further limit on reuse.
