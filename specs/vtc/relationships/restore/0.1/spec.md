---
slug: vtc/relationships/restore
version: "0.1"
title: "VTC Relationships — Restore"
summary: The issuer of a suspended relationship edge — or a moderating administrator — reverses the suspension.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: edge issuer or community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The community authorizes the act by comparing the signer with the edge's issuer, or by the signer's administrator standing, and a restoration puts an edge back into force for every reader of the relationship graph — so the signer must be attributable on every transport, and the act attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Appends a restoration to the edge's lifecycle log, reversing its standing suspension. Reversible by another suspension; the credential is untouched.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    An edge id and an optional reason in; the edge's resolved state out.
retention:
  class: durable
  rationale: >-
    The event is part of the edge's lifecycle log and is audited.
errorCodes:
  - code: vtc/relationships/restore:notFound
    meaning: "No edge with this id exists that the caller may restore. Returned identically for an unknown edge and for one the caller neither issued nor moderates."
    retryable: false
  - code: vtc/relationships/restore:notSuspended
    meaning: "The edge is not suspended. Restoration reverses a suspension and nothing else — an edge that is merely expired is restored by re-issuance, not by a lifecycle event."
    retryable: false
  - code: vtc/relationships/restore:terminal
    meaning: "The edge was superseded or withdrawn, which is terminal."
    retryable: false
related:
  - vtc/relationships/suspend
  - vtc/relationships/revoke
  - vtc/relationships/graph
---

## Abstract

The **VTC Relationships — Restore** Trust Task reverses a suspension recorded by [`vtc/relationships/suspend`](../../suspend/0.1/spec.md). It reverses a suspension and nothing else: an edge that expired, was superseded or was withdrawn is refused, because a recorded event cannot extend a window the issuer signed or undo a replacement.

Restoring an edge whose `validUntil` passed while it was suspended **succeeds** — the suspension is genuinely reversed — and the response reports `expired`, because that is the edge's resolved state.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** load the edge first, and **MUST** answer `notFound` when there is none — or when the signer is neither its issuer nor an administrator.
2. **MUST** refuse with `notSuspended` unless the edge's standing event is a suspension, and with `terminal` when it is superseded or withdrawn.
3. **MUST** read the current time once for both the appended event and the state it reports.
4. **MUST** store `reason` verbatim on the event, and **MUST** audit the restoration naming the authenticated actor and whether it acted as issuer or administrator.
5. **MUST** answer with the edge's **resolved** state after the event, which may be other than `yes`.

## Authorization

The authority this task presupposes is **control of the edge**: being its issuer, or moderating it as an administrator.

- **Issuer.** The proof's signer is the edge's `issuerDid`. For an edge published under a pairwise relationship DID, that DID signs this document directly — the document's own proof *is* the proof of control, so the separate `pop` authorization the REST surface needed (because a session DID is never the relationship DID) is not carried. An issuer's act is recorded as the issuer's.
- **Administrator.** The signer holds administrator standing at the community (a delegated signing key resolves to the identity it acts for). Moderation is keyed on the edge, not on who issued it, and is recorded as an administrator's act.

Verifying the proof establishes who asked; the comparison with the edge's issuer, or the standing, is what authorizes ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`state`** — the edge's resolved state after the restoration, an `InForce` from [`vtc/_shared/0.1/relationship-lifecycle.schema.json`](../../../_shared/0.1/relationship-lifecycle.schema.json).

## Request

The edge's issuer, or an administrator (`issuer`), names the edge to the community (`recipient`).

### The dispute is resolved

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/relationships/restore/0.1#request",
  "issuer": "did:example:relationship-did",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "4f3e2d1c-0b9a-4887-9766-5544332211aa"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Back in force

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/relationships/restore/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:relationship-did",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "4f3e2d1c-0b9a-4887-9766-5544332211aa",
    "state": {
      "state": "yes"
    }
  }
}
```

### Restored, but its window has passed

The suspension is reversed; the edge is still not in force, because it expired while suspended.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/relationships/restore/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:relationship-did",
  "issuedAt": "2026-09-28T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "id": "4f3e2d1c-0b9a-4887-9766-5544332211aa",
    "state": {
      "state": "expired",
      "validUntil": "2026-09-01T00:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An edge id and an optional reason in; the edge's resolved state out. `reason` is free text, bounded at 500 characters, written by the caller and stored on the edge's lifecycle log, where the counterparty and administrators read it; it is untrusted and attributed to its author, and a producer **SHOULD NOT** put anything in it that it would not say to the counterparty.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. An edge under a pairwise relationship DID stays pairwise: the document is signed by that DID, and the community records the act without linking it to the member's membership DID. The audit row names the authenticated actor, not the edge's issuer, because under a pairwise identifier the issuer names nobody.

### Retention

The lifecycle event is appended to the edge's log and kept with the edge. The credential itself is untouched: its signature, window and digest are the issuer's, and a lifecycle event is what this community records against it.

### Consent/purpose

The purpose is to let a party stop relying on an edge temporarily, and resume, without destroying it. Using a suspension as a judgement on the counterparty is outside it: the state says the edge is not in force here, not why.
