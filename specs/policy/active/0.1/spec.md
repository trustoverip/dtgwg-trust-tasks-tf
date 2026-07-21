---
slug: policy/active
version: "0.1"
title: Policy — Active
summary: Read the policy currently active for a purpose decision slot — the active binding for one purpose, or every active binding. The read companion to policy/activate.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - rego
  - active
  - governance
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
  rationale: Read-only inventory of the active bindings. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Read-only; returns the current active bindings and persists nothing."
subjectPath: /purpose
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: policy/active:permissionDenied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
---

## Abstract

The **Policy — Active** Trust Task reads the active-policy bindings — the `(contextId, purpose) → policy` map that [`policy/activate`](../../activate/0.1/) writes. It is the read side of that write: given a `purpose`, it returns the single policy currently authoritative for that slot (or nothing, if none is active); with `purpose` omitted, it returns every active binding.

It exists as its own task rather than a filter on [`policy/list`](../../list/0.2/) because a purpose is **not** a property of a policy in this model — it is a binding produced by activation (see `policy/activate`). `policy/list` enumerates policy *modules* (which carry `appliesTo`/`priority`, not `purpose`); this task enumerates the *bindings*. A `purpose` filter on `list` would be answering a question its result shape cannot express.

## Conformance

Producer: optionally supply `purpose` to narrow to one slot, and `contextId` to scope to one context.

Consumer: verify `PolicyAdmin` capability (the same gate `policy/list` applies). Return one `ActiveBinding` per matching active slot, each carrying the full `PolicyModule` so the caller need not follow up with `policy/get`. When `purpose` is supplied and no policy is active for it, return an empty `bindings` array — not an error; absence of an active policy is a normal state, not a lookup failure. The set is unpaginated: there is at most one active policy per purpose, so the active set is small by construction.

## Security & Privacy

**Source disclosure.** Each binding carries the active policy's `PolicyModule`, including its Rego `module`. As with `policy/list`, Rego source is not a secret but describes the maintainer's security posture and SHOULD be visible only to admin-class consumers — the `PolicyAdmin` gate enforces that.

**No existence oracle over purposes.** An unknown or unbound `purpose` returns an empty array, identical to a known-but-inactive one, so this task does not distinguish "purpose never used" from "purpose has no active policy". That is intentional: `purpose` is a free maintainer-scoped string, and there is nothing to enumerate against.

**Auditing.** Recommended at sampled rate, matching `policy/list`.
