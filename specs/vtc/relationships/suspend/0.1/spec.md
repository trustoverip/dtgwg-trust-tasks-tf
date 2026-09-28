---
slug: vtc/relationships/suspend
version: "0.1"
title: "VTC Relationships — Suspend"
summary: The issuer of a published relationship edge — or a moderating administrator — makes it temporarily ineffective without withdrawing it, leaving a supported way back.
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
    The community authorizes the act by comparing the signer with the edge's issuer, or by the signer's administrator standing, and a suspension changes what every reader of the relationship graph relies on — so the signer must be attributable on every transport, and the act attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Appends a suspension to the edge's lifecycle log; the edge stops being in force here. Reversible by vtc/relationships/restore, and the credential itself is untouched — which is what distinguishes it from vtc/relationships/revoke, which deletes the edge.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    An edge id and an optional reason in; the edge's resolved state out. The reason is free text the caller wrote.
retention:
  class: durable
  rationale: >-
    The event is part of the edge's lifecycle log and is audited; readers of the graph rely on it until it is reversed.
errorCodes:
  - code: vtc/relationships/suspend:notFound
    meaning: "No edge with this id exists that the caller may suspend. Returned identically for an unknown edge and for one the caller neither issued nor moderates, so the code cannot be used to probe for others' edges."
    retryable: false
  - code: vtc/relationships/suspend:alreadySuspended
    meaning: "The edge is already suspended. A second suspension records nothing a reader could act on."
    retryable: false
  - code: vtc/relationships/suspend:terminal
    meaning: "The edge was superseded or withdrawn, which is terminal; it is given effect again by issuing and publishing a fresh credential, not by a lifecycle event."
    retryable: false
related:
  - vtc/relationships/restore
  - vtc/relationships/revoke
  - vtc/relationships/list
  - vtc/relationships/graph
---

## Abstract

A published relationship edge had exactly two states — published and deleted — so a party with a reason to stop relying on one *temporarily* had to destroy it, and its issuer had to re-issue and re-publish to get it back. That is not a smaller revocation; it is a different act.

The **VTC Relationships — Suspend** Trust Task records it: the edge stays published, its credential untouched, and a suspension on its lifecycle log makes it not in force here until [`vtc/relationships/restore`](../../restore/0.1/spec.md) reverses it. Readers of the graph ([`vtc/relationships/graph`](../../graph/0.2/spec.md)) see a suspended half as not consenting.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** load the edge first, and **MUST** answer `notFound` when there is none — or when the signer is neither its issuer nor an administrator.
2. **MUST** refuse with `alreadySuspended` when the edge's standing event is a suspension, and with `terminal` when it is superseded or withdrawn.
3. **MUST** read the current time once and use it both for the appended event and for the state it reports, so the two cannot straddle two instants.
4. **MUST** store `reason` verbatim on the event, and **MUST** audit the suspension naming the authenticated actor and whether it acted as issuer or administrator.
5. **MUST** answer with the edge's **resolved** state after the event.

## Authorization

The authority this task presupposes is **control of the edge**: being its issuer, or moderating it as an administrator.

- **Issuer.** The proof's signer is the edge's `issuerDid`. For an edge published under a pairwise relationship DID, that DID signs this document directly — the document's own proof *is* the proof of control, so the separate `pop` authorization the REST surface needed (because a session DID is never the relationship DID) is not carried. An issuer's act is recorded as the issuer's.
- **Administrator.** The signer holds administrator standing at the community (a delegated signing key resolves to the identity it acts for). Moderation is keyed on the edge, not on who issued it, and is recorded as an administrator's act.

Verifying the proof establishes who asked; the comparison with the edge's issuer, or the standing, is what authorizes ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **Edge** — one published Verifiable Relationship Credential.
- **Lifecycle log** — the append-only record of events this community holds against an edge.
- **`state`** — the edge's resolved state, an `InForce` from [`vtc/_shared/0.1/relationship-lifecycle.schema.json`](../../../_shared/0.1/relationship-lifecycle.schema.json).

## Request

The edge's issuer, or an administrator (`issuer`), names the edge to the community (`recipient`).

### An issuer suspends an edge while a dispute is resolved

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/relationships/suspend/0.1#request",
  "issuer": "did:example:relationship-did",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "4f3e2d1c-0b9a-4887-9766-5544332211aa",
    "reason": "Paused while we sort out the workshop booking."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the edge id and its resolved state. A refusal is a `trust-task-error`.

### Suspended

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/relationships/suspend/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:relationship-did",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "4f3e2d1c-0b9a-4887-9766-5544332211aa",
    "state": {
      "state": "suspended",
      "since": "2026-09-28T10:00:01Z"
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
