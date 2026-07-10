---
slug: auth/step-up/policy
version: "0.2"
title: Auth — Step-up Policy
summary: The relying party's per-operation-class policy deciding whether — and how — a session must step up to a higher assurance level before a gated operation runs, plus how a system-wide floor composes with per-entry overrides.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - auth
  - step-up
  - aal
  - policy
  - access-control
  - authorization
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Setting this policy changes the security posture of every gated operation on the maintainer. Without a verified proof an attacker holding a single captured token could weaken or disable the step-up gate it is meant to defend, then proceed unchallenged.
sideEffects:
  level: mutating
  rationale: "Declares the per-operation-class step-up policy the relying party enforces; changes gating for subsequent operations."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/step-up/policy:notAuthorized
    meaning: The issuer is not authorized to set the maintainer's step-up policy.
    retryable: false
  - code: auth/step-up/policy:unknownOperation
    meaning: A floor names an operation-class the maintainer does not recognize or does not gate.
    retryable: false
  - code: auth/step-up/policy:lockoutRefused
    meaning: The requested policy would enable enforcement for an operation-class while leaving no party able to satisfy it, locking the maintainer's administrators out. The maintainer refuses rather than apply a self-lockout.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - acl/grant
  - acl/swap-key
---

## Abstract

The **Auth — Step-up Policy** Trust Task carries a relying party's (an ACL maintainer's) decision about *when* a session must elevate to a higher Authenticator Assurance Level (AAL) before a gated operation runs, and *how* that elevation is ratified. It is the policy half of the step-up family: [`auth/step-up/approve-request`](../../approve-request/0.1/spec.md) / [`auth/step-up/approve-response`](../../approve-response/0.1/spec.md) are the *mechanism* by which a step-up is performed; this document is the *policy* that decides whether to demand one and who may approve it.

A maintainer holds one step-up policy. The policy is a per-**operation-class** *floor* — a minimum required mode. A floor composes with per-entry overrides carried on each subject's `AclEntry.stepUp` (see [`acl/_shared`](../../../../acl/_shared/0.1/acl-entry.schema.json)): an override MAY make the requirement **stricter** for that subject, never weaker. This lets an operator say *"deleting any context requires a delegated approval, system-wide"* while still allowing a sensitive context to demand more.

The design resolves a bootstrapping tension. A freshly-provisioned maintainer has no registered approver, so enforcing step-up out of the box would brick every gated operation — including the operations needed to register the first approver. The policy therefore ships **disabled** (AAL1 everywhere) and is promoted deliberately, with guardrails (fail-closed on escalation, a non-escalation carve-out for rotation/enrolment, and a refusal to apply a self-lockout).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/policy/0.2`, with the administrator as `issuer` and the maintainer as `recipient`.
2. Populate `payload.enabled` and `payload.floors`.
3. Include a verified `proof`; the maintainer relies on it to authorize the mutation.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Verify the document's `proof` and that the `issuer` is authorized to set policy; otherwise reject with `notAuthorized`.
2. Reject with `unknownOperation` if a `floor.operation` is neither `*` nor an operation-class the maintainer gates.
3. **Refuse self-lockout.** If applying the policy would enable enforcement (`enabled: true`) for an operation-class whose resolved mode requires AAL2, while **no** party currently holds a usable method to satisfy it (no registered `approver`, no self authenticator), the maintainer **MUST** reject with `lockoutRefused` and leave the prior policy in force. Enabling enforcement is the moment to prove an approver exists.
4. Apply the policy atomically and return a `#response` carrying the effective (canonicalized) policy.
5. **Default posture.** Until a policy is set, the maintainer **MUST** behave as `enabled: false` (AAL1 everywhere) and **SHOULD** surface the not-enforced state prominently to operators.
6. Enforce the **resolution algorithm** (below) on every gated operation.
7. Retain an **out-of-band administrative path** to read and roll back this policy that does **not** traverse the step-up gate (see Security & Privacy → *Break-glass*).

### Resolution algorithm (normative)

For a gated operation `op` requested by caller `c` whose `AclEntry` is `e`:

1. If `policy.enabled` is `false` → **mode = `none`**. Proceed at AAL1.
2. Otherwise select the **floor**: the `floor` whose `operation` matches `op` most specifically, else the `*` floor, else `none`.
3. Compute **effective mode** = the *strictest* of `floor.mode` and `e.stepUp.require` (absent override ⇒ just `floor.mode`). Strictness order: `none` < `self` < `delegatedAny` < `delegated`. An override weaker than the floor is ignored (additive-only).
4. If effective mode is `none` → proceed at AAL1.
5. Otherwise AAL2 is required. If the caller's session already satisfies AAL2 → proceed.
6. Otherwise resolve a **method**:
   - `delegated` → the `approver` on `e.stepUp`. `self` → the caller's own authenticator. `delegatedAny` → any VID meeting the maintainer's approver criterion.
   - If a usable method exists → issue an `auth/step-up/approve-request` addressed to the resolved approver and gate the operation on a valid `approve-response`.
   - If **no** usable method exists:
     - If the matching floor sets `allowAal1IfNonEscalating: true` **and** the maintainer verifies the request is **non-escalating** (see below) → admit at AAL1.
     - Otherwise **fail closed**: reject the operation. The maintainer MUST NOT silently downgrade to AAL1.

### Non-escalation check (normative)

A request is **non-escalating** when **all** hold: (a) the caller acts on its own `AclEntry` (the entry's `subject` is the caller, or — for `acl/swap-key` — `currentSubject` equals the caller); (b) the resulting entry's `role` is unchanged; and (c) the resulting entry's `scopes` are a subset of the caller's existing `scopes`. Key-rotation (`acl/swap-key`) and step-up-method enrolment are the canonical non-escalating self-service operations. Any request that grants a new role, widens scopes, or acts on another subject's entry is **escalating** and is never eligible for the AAL1 carve-out.

## Definitions

* **Operation-class.** A family of gated operations identified by a Trust Task type/slug (e.g. `acl/grant`), or `*` for the default.
* **Mode.** The required step-up shape: `none` (AAL1) | `self` | `delegated` | `delegatedAny`.
* **Floor.** The maintainer's system-wide minimum mode for an operation-class.
* **Override.** A per-entry `stepUp` setting (on the subject's `AclEntry`) that may raise — never lower — the floor for that subject.
* **Approver.** The party that ratifies a step-up; for `delegated`, the `approver` VID on the caller's entry. See [`auth/step-up/approve-request`](../../approve-request/0.1/spec.md).
* **Non-escalating.** A self-service request that does not increase the caller's authority — see the check above.

## Request

The **administrator** sends the policy to the **maintainer**. The top-level schema in [`payload.schema.json`](./payload.schema.json) describes the request payload.

### Enable enforcement with a system-wide floor

A maintainer that has already registered an approver promotes its posture: ACL grants and high-risk deletes require a delegated approval; key-rotation stays self-service at AAL1 via the non-escalation carve-out.

```json
{
  "id": "stepup-policy-7f1c-4e2a-9b3d-0c5e6a1b2c3d",
  "type": "https://trusttasks.org/spec/auth/step-up/policy/0.2",
  "issuer": "did:web:admin.acme.example",
  "recipient": "did:web:vta.acme.example",
  "issuedAt": "2026-06-01T09:00:00Z",
  "payload": {
    "enabled": true,
    "floors": [
      { "operation": "*", "mode": "self" },
      { "operation": "acl/grant", "mode": "delegated" },
      { "operation": "acl/change-role", "mode": "delegated" },
      { "operation": "acl/revoke", "mode": "delegated" },
      { "operation": "context/delete", "mode": "delegated" },
      { "operation": "key/revoke", "mode": "delegated" },
      { "operation": "acl/swap-key", "mode": "self", "allowAal1IfNonEscalating": true }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.acme.example#key-1",
    "created": "2026-06-01T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Disabled — the shipping default (break-glass rollback)

The form an operator applies from the local console to recover from an over-strict policy. Disabling reverts to AAL1 everywhere.

```json
{
  "id": "stepup-policy-1a2b-3c4d-5e6f-7a8b9c0d1e2f",
  "type": "https://trusttasks.org/spec/auth/step-up/policy/0.2",
  "issuer": "did:web:admin.acme.example",
  "recipient": "did:web:vta.acme.example",
  "issuedAt": "2026-06-01T09:05:00Z",
  "payload": { "enabled": false, "floors": [] },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.acme.example#key-1",
    "created": "2026-06-01T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

## Response

The **maintainer** returns the effective policy it now holds — `floors` canonicalized (deduplicated by `operation`, defaults materialized). The sub-schema is reachable via `$anchor: "response"` in [`payload.schema.json`](./payload.schema.json). Failures use `trust-task-error` (e.g. `lockoutRefused`), not a `#response`.

```json
{
  "id": "stepup-policy-resp-9c0d-1e2f-3a4b-5c6d7e8f9a0b",
  "type": "https://trusttasks.org/spec/auth/step-up/policy/0.2#response",
  "threadId": "stepup-policy-7f1c-4e2a-9b3d-0c5e6a1b2c3d",
  "issuer": "did:web:vta.acme.example",
  "recipient": "did:web:admin.acme.example",
  "issuedAt": "2026-06-01T09:00:01Z",
  "payload": {
    "enabled": true,
    "floors": [
      { "operation": "*", "mode": "self", "allowAal1IfNonEscalating": false },
      { "operation": "acl/grant", "mode": "delegated", "allowAal1IfNonEscalating": false },
      { "operation": "acl/change-role", "mode": "delegated", "allowAal1IfNonEscalating": false },
      { "operation": "acl/revoke", "mode": "delegated", "allowAal1IfNonEscalating": false },
      { "operation": "context/delete", "mode": "delegated", "allowAal1IfNonEscalating": false },
      { "operation": "key/revoke", "mode": "delegated", "allowAal1IfNonEscalating": false },
      { "operation": "acl/swap-key", "mode": "self", "allowAal1IfNonEscalating": true }
    ]
  }
}
```

### Failure — refused self-lockout

```json
{
  "id": "tterr-2b3c-4d5e-6f7a-8b9c0d1e2f3a",
  "type": "https://trusttasks.org/spec/trust-task-error/0.2",
  "threadId": "stepup-policy-7f1c-4e2a-9b3d-0c5e6a1b2c3d",
  "issuer": "did:web:vta.acme.example",
  "recipient": "did:web:admin.acme.example",
  "issuedAt": "2026-06-01T09:00:01Z",
  "payload": {
    "code": "auth/step-up/policy:lockoutRefused",
    "message": "Enabling 'delegated' for acl/grant would lock out all administrators: no AclEntry carries a stepUp.approver. Register an approver, then enable."
  }
}
```

## Security & Privacy

**Fail-closed, not fail-open.** When a policy requires AAL2 for an operation and the caller has no usable method, the maintainer denies the operation. Resolving "no method available" to AAL1 would be a downgrade vector: a control that silently disables itself when its prerequisite is absent protects nothing. The only exception is the explicit, per-floor `allowAal1IfNonEscalating` carve-out, gated on a server-verified non-escalation check — it admits a holder's own non-escalating rotation/enrolment but never a privilege grant.

**Bootstrapping vs steady state.** Shipping `enabled: false` is a deliberate, *documented-loud* open posture, not a default-insecure accident — the maintainer cannot register its first approver while enforcing a gate that requires one. The window in which a methodless principal may perform a non-escalating swap-key at AAL1 is bounded: once it enrols an authenticator, the floor (`self`/`delegated`) is enforced for it. The residual risk during that window — a thief holding the principal's AAL1 token swapping the entry to a key they control — is accepted only because it is bounded to pre-enrolment and the swap still requires proof of control of the new key.

**Additive-only overrides.** Per-entry `stepUp` settings may only raise the requirement. A consumer MUST ignore an override weaker than the resolved floor, so a compromised or misconfigured entry cannot weaken a system-wide protection.

**Anti-lockout.** Enabling enforcement is the one operation that can brick the maintainer. Consumers MUST refuse a policy that would leave an operation-class enforced with no party able to satisfy it (`lockoutRefused`), forcing the operator to register an approver first.

**Break-glass.** This policy governs **wire-facing** access control only. A maintainer MUST retain an out-of-band administrative path — available to an operator with direct, locally-trusted control of the maintainer's key material — to read and roll back this policy **without** traversing the step-up gate, so an over-strict policy can be recovered even when every remote credential is locked out. Implementations differ by deployment: a local administrative CLI with direct store access is one form; deployments that intentionally expose **no** local operator backdoor (e.g. confidential-computing / TEE maintainers) MUST instead provide a re-provisioning or emergency-recovery ceremony, since for those the wire is the only channel and a self-lockout is otherwise terminal.

**Proof integrity.** Setting policy is a posture-changing mutation; the `proof` (REQUIRED) binds the request to an authorized administrator. Consumers MUST verify it before applying — an unauthenticated policy set is a direct path to disabling the gate.

The optional `ext` extension is part of the signed surface.
