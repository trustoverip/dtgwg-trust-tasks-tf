---
slug: auth/step-up/approver/revoke
version: "0.1"
title: "Auth — Step-up Approver Revoke"
summary: Revoke one step-up approver binding, so the next statement the approver signs is refused — by the subject it is bound to, or by an administrator acting on their behalf.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - revocation
  - second-factor
parties:
  - role: Subject, or an administrator
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
subjectPath: /subject
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The relying party decides whether the caller may revoke by comparing the signer's resolved identity with the binding's subject, or with an administrator's standing over that subject, so the signer has to be attributable on every transport. A revocation is also the record an operator points to after an incident, which a proof makes durable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay changes nothing — a revoked approver stays revoked — but §7.2 item 11 still needs a bounded window to recognise the duplicate.
sideEffects:
  level: destructive
  rationale: >-
    Revocation is irreversible: the binding becomes a tombstone and the same approver DID can never be bound again, to this subject or any other. It removes a factor and confers nothing; the subject's authority is untouched, and a subject left with no factor recovers through an administrator's invite.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names an approver, optionally its subject, and an optional free-text reason; the response returns the revoked binding, when it was revoked and how many approvers the subject still holds. No key material either way.
retention:
  class: durable
  rationale: >-
    The tombstone is kept so the approver cannot be re-bound and so the subject can see that a factor was disowned. The revocation, with its reason, is audited against the caller.
errorCodes:
  - code: auth/step-up/approver/revoke:notFound
    meaning: "No binding of this approver exists that the caller may revoke. Returned identically for an approver that was never bound, one bound to a different subject than `subject` names, and one bound to a subject the caller has no authority over, so the code cannot be used to probe which approvers are bound to whom."
    retryable: false
related:
  - auth/step-up/approver/list
  - auth/step-up/approver/enroll
  - auth/step-up/approver/invite
  - auth/step-up/approver/redeem/finish
---

## Abstract

The **Auth — Step-up Approver Revoke** Trust Task withdraws one step-up approver binding. From the moment it executes, a statement signed by that approver is refused for every step-up, and the approver DID cannot be bound again.

A subject revokes their own approver when a device is lost or a browser profile is suspect. An administrator revokes a member's — naming `subject` — for incident response, or when the member reports a lost device and cannot reach any factor themselves.

**Revoking the last approver is allowed.** It costs the subject the ability to answer a step-up with an approver, and nothing else: their authority, their DID and their other factors are untouched, and it is recoverable — through another factor they hold with [`enroll`](../../enroll/0.1/spec.md), or through an administrator's [`invite`](../../invite/0.1/spec.md). Refusing it would leave a subject unable to disown a factor they believe compromised, which protects the attacker.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the relying party):

1. **MUST** resolve the proof's signer to an identity — the signer itself, or the identity a delegated key acts for.
2. **MUST** treat an absent `subject` as that identity, and require the approver to be bound to the subject — `notFound` otherwise.
3. **MUST** accept a revocation for the caller's own approvers, and **MUST** accept one for another subject's only when the caller holds administrative standing that covers that subject; `notFound` otherwise. A step-up approver never revokes anything: a statement it signed is not a revocation, and its DID is never a caller here.
4. **MUST** leave a tombstone rather than delete the binding, so the approver DID cannot be bound again.
5. **MUST** succeed, changing nothing, for an approver already revoked from that subject, and report its original `revokedAt`.
6. **MUST** serialise the revocation with every other write of the subject's approvers, so it cannot interleave with an enrolment or a rotation that names it.
7. **MUST** report `remainingApprovers` — the subject's approvers still live afterwards — and **MUST** audit the revocation against the caller, naming the approver, the subject and the `reason` when one was given.
8. **SHOULD** tell the subject, over a channel it already has with them, when an administrator revoked one of their approvers.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is **the subject's standing over their own factors**, or **an administrator's standing over the subject** at the relying party. Both are read from the relying party's own state; the proof establishes who asked, never that they may.

The reference implementation — a VTC — gates a revocation behind an operation-bound step-up satisfied by the caller's own factor, as it does an enrolment, so that a stolen signing key alone cannot strip a subject of the factors that protect them. That gate is the relying party's policy. A subject whose only factor is the one being revoked can still answer it with that factor; a subject who holds none can ask an administrator, whose own authority answers instead.

## Definitions

- **`approverDid`** — the approver to revoke.
- **`subject`** — the subject the approver is bound to; the caller when absent. An administrator revoking on someone's behalf names it.
- **`reason`** — why, for the audit record. Free text.
- **`revoked`** — the binding as it stood when revoked.
- **`revokedAt`** — when it was revoked; the original time for a repeated revocation.
- **`remainingApprovers`** — how many live approvers the subject holds afterwards. Zero is legitimate.

## Request

The caller (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator revokes a member's approver after a lost laptop

```json
{
  "id": "urn:uuid:c3f7a0b4-5d8e-4f90-b12c-3d4e5f6a7b01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/revoke/0.1",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T12:00:00Z",
  "threadId": "urn:uuid:c3f7a0b4-5d8e-4f90-b12c-3d4e5f6a7bff",
  "payload": {
    "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
    "reason": "Laptop reported lost"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid8:acme-vtc.example:dana#key-1",
    "created": "2026-10-02T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`. Refusals use `trust-task-error`.

### The approver is revoked; Alice has none left

```json
{
  "id": "urn:uuid:c3f7a0b4-5d8e-4f90-b12c-3d4e5f6a7b02",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/revoke/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-10-02T12:00:01Z",
  "threadId": "urn:uuid:c3f7a0b4-5d8e-4f90-b12c-3d4e5f6a7bff",
  "payload": {
    "revoked": {
      "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
      "label": "Browser plugin — work laptop",
      "enrolledAt": "2026-10-01T15:04:06Z",
      "enrolledVia": "invite",
      "lastUsedAt": "2026-10-02T09:00:06Z"
    },
    "revokedAt": "2026-10-02T12:00:01Z",
    "remainingApprovers": 0
  }
}
```

## Security & Privacy

**Withdrawal, not escalation.** Revocation removes a factor and confers nothing, so its abuse is denial of service — a subject made unable to answer a step-up until they enrol another factor. That is why the authority is the subject's own, or an administrator's over them, and why a caller with no standing over the subject is answered `notFound`.

**A revoked approver stays revoked.** A factor revoked as compromised must not come back because someone later obtained an invite; the tombstone is what prevents it.

**Free text.** `reason` is bounded at 500 characters, authored by the caller and untrusted. It is kept in the audit record and shown to the subject and administrators; it is never published beyond them, and a surface rendering it **MUST** attribute it to the caller. A caller **SHOULD** say which condition applied ("device lost") rather than describe a person.

### Data carried

The request carries an approver DID, optionally a subject DID and a reason. The response carries the revoked binding, a timestamp and a count. No key material.

### Correlation

The revocation links the caller to the subject and the approver at one moment. The approver DID is used at this relying party only, so the revocation reveals nothing elsewhere.

### Retention

The tombstone is kept for as long as the relying party keeps the subject's factors, so the approver cannot be re-bound. The audit record of the revocation, with its reason, is durable.

### Consent/purpose

The task exists to disown one factor. Its `reason` is collected for the audit record of that act and **MUST NOT** be reused for anything else. Whether the relying party asks for a step-up before revoking is its own policy.
