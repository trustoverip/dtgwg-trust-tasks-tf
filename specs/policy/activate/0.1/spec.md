---
slug: policy/activate
version: "0.1"
title: Policy — Activate
summary: Make a policy module the single active policy for a named decision slot (purpose), atomically deactivating whatever was active before and returning the displaced id.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - rego
  - activate
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
  requirement: REQUIRED
  rationale: Activation changes which policy governs a decision slot — a different active `join` policy admits different members. The producer's identity MUST be verifiable for audit and to prevent a stealth swap of the authoritative policy.
sideEffects:
  level: mutating
  rationale: "Changes the active policy for a purpose; the previously-active policy is deactivated. Reversible by activating the previous id (returned as previousPolicyId)."
consequences:
  - "Changes the policy evaluated for this purpose on every subsequent request, effective immediately."
  - "Deactivates the policy previously active for the same (contextId, purpose)."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: policy/activate:notFound
    meaning: No policy module with the supplied `id` exists.
    retryable: false
  - code: policy/activate:policyDisabled
    meaning: The policy exists but is disabled (its `enabled` flag is false); a disabled policy cannot be activated. Enable it first via policy/upsert.
    retryable: false
  - code: policy/activate:alreadyActive
    meaning: The supplied `id` is already the active policy for this (contextId, purpose). Refused so the audit log carries no no-op activation.
    retryable: false
---

## Abstract

The **Policy — Activate** Trust Task selects one policy module as the single authoritative policy for a **purpose** — a named decision slot such as a community governance stage (`join`, `removal`, …). This is a different selection model from the `appliesTo` / `priority` layering in [`policy/_shared`](../../_shared/0.3/policy.schema.json): there, many enabled policies matching a context are evaluated in priority order and the first non-`null` decision wins; here, exactly one policy is active per `(contextId, purpose)`, and it alone decides that slot. The two coexist — a maintainer that never activates a purpose uses pure priority ordering; one that does uses the active policy for that purpose.

Activation is **relational, not intrinsic**: a policy is not born tied to a purpose, it is bound to one here. A maintainer MAY activate the same module for more than one purpose, and a consumer that wants a policy to declare its own intended purpose carries that in a vendor `ext` namespace — the framework does not model it. `purpose` is an opaque maintainer-scoped string; the framework does not enumerate its values.

The `previousPolicyId` in the response is what makes the swap auditable and reversible: it names the policy this call displaced, so an operator can roll back by activating it again and the audit trail links the two.

## Conformance

Producer: supply `id` and `purpose`. Supply `contextId` only if the maintainer partitions its active-policy map per context. Carry a proof.

Consumer: verify `PolicyAdmin` capability. Resolve the policy by `id`; if none exists, return `notFound`. If it is `enabled: false`, return `policyDisabled`. If it is already the active policy for `(contextId, purpose)`, return `alreadyActive` — do not re-emit audit or sync events for a no-op. Otherwise **atomically** replace the active pointer for `(contextId, purpose)`: record the displaced id, set the new one, and return `previousPolicyId` (or `null` on the first activation for the slot). Emit `sync/event/0.1` with kind `policyChanged`, and audit the swap with `{ who, when, purpose, contextId, activated, previousPolicyId }`.

## Security & Privacy

**Posture change.** Activation silently redirects every subsequent decision for the purpose to a different policy — higher-stakes than authoring a policy that is not yet active. Hence `proofRequirement: REQUIRED` and mandatory audit of the swap.

**Atomic swap under contention.** Two concurrent activations for the same `(contextId, purpose)` MUST NOT interleave: the read-displaced-then-set sequence runs under a per-slot lock so exactly one wins and the loser sees the winner as `previousPolicyId`. Without this, both could report the same `previousPolicyId` and the active pointer could end on either — the audit trail would then misrepresent the order.

**No-op suppression.** Re-activating the already-active policy returns `alreadyActive` rather than succeeding, so the audit log cannot be padded with activations that changed nothing — a reviewer reading the trail sees only real transitions.

**Free text.** `purpose` names a decision slot and is constrained by no
vocabulary the framework defines — each maintainer defines its own set — so it
is free text under [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item
19. The request member already declared `maxLength: 128`; the response member
that echoes it now declares the same bound, because a value that fits the
request must fit the reply. It is authored by the operator activating the
policy, read by the maintainer's own policy router and by anyone reading the
active-policy map, and the maintainer **retains** it for as long as the binding
is active — it is half the binding's key. A maintainer SHOULD choose slot names
that name a decision rather than a subject: this value is stable and widely
read, which makes it a poor place for anything about a person.

