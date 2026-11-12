---
slug: messaging/mux/admin-list
version: "0.1"
title: "Messaging — Mux Admin List"
summary: "An administrator lists the stored mux grants of a principal DID or of a member DID a page at a time — each grant's parties, state, scope, expiry and epoch, and the member's current epoch — without the signed statements."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, list, admin, operator, console]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request is gated on administrative standing and returns delegations between DIDs other than the requester's; a proof binds the read to the administrator's key independent of the transport, so the mediator can show who looked. The response is RECOMMENDED so an operator can keep the page that justified a revocation as evidence of what was in force.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on every administrative request; without issuedAt a captured request could be replayed indefinitely to watch a DID's delegations change.
sideEffects:
  level: none
  rationale: >-
    Reads the mediator's grant records and a member DID's epoch; changes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Discloses delegation metadata to an administrator: for each stored grant of the named principal or member DID, its identifier, its two parties, state, effective scope and expiry, the epoch it was signed under and when, and the member's current epoch. Never the signed statement, its proof, a nonce, key material or message content. The request carries a principal or member DID.
retention:
  class: transient
  rationale: >-
    A page describes grants that a revocation, an expiry or a new grant can change at any moment; it is superseded by the next read.
maxDocumentBytes:
  response: 2097152
  rationale: >-
    A page carries at most 500 grants, each two DIDs of up to 2048 characters and a few short members.
errorCodes:
  - code: messaging/mux/admin-list:cursorInvalid
    meaning: '`cursor` was not issued by this mediator for this requester and these request members, or it has expired.'
    retryable: false
related:
  - messaging/mux/admin-revoke
  - messaging/mux/admin-revoke-all
  - messaging/mux/connections
  - messaging/mux/list
  - messaging/mux/grant
---

## Abstract

The **Messaging — Mux Admin List** Trust Task gives a mediator's operator the stored side of DID multiplexing. [`messaging/mux/list`](../../list/0.1/spec.md) shows a delegation to its two parties only, and [`messaging/mux/connections`](../../connections/0.1/spec.md) shows the live connections and the members riding them now. This task lists the **stored grants** ([`messaging/mux/grant`](../../grant/0.1/spec.md)) of one principal DID, or of one member DID, in every state — whether or not the principal is connected — with each grant's parties, state, effective scope and expiry, and the epoch it was signed under.

It answers the operator's questions before an administrative revocation: *which members does this principal hold grants from?* (filter by `principal`), *which principals hold grants on this DID, and what is its epoch?* (filter by `member`), and, with both, *what does this principal hold of this member?* The grants it shows are what [`messaging/mux/admin-revoke`](../../admin-revoke/0.1/spec.md) revokes, and the epoch it reports is what [`messaging/mux/admin-revoke-all`](../../admin-revoke-all/0.1/spec.md) advances.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.6 and §7.8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/mux/admin-list/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`, and naming at least one of `principal` and `member`. To continue a listing it **MUST** send the previous page's `nextCursor` verbatim with the same `principal` and `member`.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `malformedRequest` when neither `principal` nor `member` is present.
2. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
3. Respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
4. List the stored grants whose principal is `principal` and whose member DID is `member`, for each of the two that is present — active grants, and dead ones still within the retention window after their effective expiry — ordered by member DID, then principal DID, then grant identifier.
5. Give each grant its effective scope and expiry, its state as [`messaging/mux/list`](../../list/0.1/spec.md) would report it (an `expired` grant is computed when read), the epoch its statement signs and its signed `iat` as `signedAt`, and `narrowed: true` where [`messaging/mux/update`](../../update/0.1/spec.md) narrowed it below what the statement signs.
6. Include `epoch`, the member DID's current epoch, exactly when `member` is present — `0` for a member DID whose epoch has never advanced.
7. Bind `nextCursor` to the requester and the other request members, and refuse any other cursor, or one it has expired, with `messaging/mux/admin-list:cursorInvalid`. Omit `nextCursor` on the last page.
8. Never include a grant's signed statement or its proof, a connection's `muxNonce`, a message, or any key material.

A filter that matches nothing is an empty page, not an error: a DID with no grants, and a DID with no account at the mediator, both list nothing.

This task is a read-many with no `show` sibling, for the reason [`messaging/mux/list`](../../list/0.1/spec.md#conformance) gives: a grant identifier is chosen by its member and unique only on that member's record, so a grant is reached through its member or its principal, and both filters are here. The tasks that act on one grant — [`messaging/mux/admin-revoke`](../../admin-revoke/0.1/spec.md) among them — already answer an absent grant with a definite determination ([CONTRIBUTING-SPECS "Read-one and read-many"](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)).

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin`, `rootAdmin`, or `mediator` role (Consumer rule 3), as for [`messaging/mux/connections`](../../connections/0.1/spec.md). The listing names principals and member DIDs other than the requester's, so there is no self-service form: a principal or a member DID reads its own delegations with [`messaging/mux/list`](../../list/0.1/spec.md), which also shows it the signed statements this task withholds.

The `proof` establishes *who* asked and that the request is unaltered, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the mediator resolves the verified issuer to its account and decides from the role it finds there.

## Definitions

- **`principal`** — OPTIONAL. Only grants delegated to this principal DID ([`MuxDid`](../../_shared/0.1/mux.schema.json#/$defs/MuxDid)).
- **`member`** — OPTIONAL. Only grants on this member DID. At least one of `principal` and `member` is REQUIRED; with both, only the grants between the two.
- **`cursor`**, **`limit`** — OPTIONAL paging: an opaque token from `nextCursor`, and the most grants to return (1–500; default 100).
- **`grants`** (response) — `AdminGrantView`s: `grantId` ([`GrantId`](../../_shared/0.1/mux.schema.json#/$defs/GrantId)), `did` (the member DID), `principal`, `state` ([`GrantState`](../../_shared/0.1/mux.schema.json#/$defs/GrantState)), `scope` ([`MuxScope`](../../_shared/0.1/mux.schema.json#/$defs/MuxScope)), `expiresAt`, `epoch`, `signedAt` and `narrowed`.
- **`epoch`** (response) — with `member` only: the member DID's current epoch.
- **`nextCursor`** (response) — absent on the last page.

An `AdminGrantView` is deliberately not a [`GrantView`](../../_shared/0.1/mux.schema.json#/$defs/GrantView): the signed statement in a `GrantView` is evidence its two parties may keep and re-verify, and an operator needs what it delegates, not a copy of the member's signature.

## Request

The administrator sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Which principals hold grants on a DID?

```json
{
  "id": "urn:uuid:7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-list/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-13T09:00:00Z",
  "payload": {
    "member": "did:webvh:QmA1ice:alice.example:work"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-13T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The mediator responds with a page of grants, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) rather than a `#response` document.

### Two principals, one grant suspended

```json
{
  "id": "urn:uuid:7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c02",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-list/0.1#response",
  "threadId": "urn:uuid:7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:00:00Z",
  "payload": {
    "grants": [
      {
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "state": "suspended",
        "scope": ["receive", "pickup"],
        "expiresAt": "2026-12-09T09:00:00Z",
        "epoch": 3,
        "signedAt": "2026-11-09T09:00:00Z"
      },
      {
        "grantId": "5d6e7f80-9a1b-4c2d-8e3f-4a5b6c7d8e9f",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmTab1et:tablet.example",
        "state": "active",
        "scope": ["receive"],
        "expiresAt": "2026-11-20T09:00:00Z",
        "epoch": 3,
        "signedAt": "2026-11-12T08:30:00Z",
        "narrowed": true
      }
    ],
    "epoch": 3
  }
}
```

### Refused: not an administrator

```json
{
  "id": "urn:uuid:7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:00:00Z",
  "payload": {
    "code": "permissionDenied",
    "message": "messaging/mux/admin-list requires an administrator account.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The response carries, per stored grant, its identifier, member and principal DIDs, state, effective scope and expiry, the epoch it was signed under and its signing time, and, with `member`, that DID's current epoch. It carries no signed statement and no proof — those stay with [`messaging/mux/list`](../../list/0.1/spec.md), which shows them to their parties — no connection nonce, no message content and no key material. The request carries a principal DID, a member DID, or both, and paging members. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The listing is a map of which DIDs a principal holds, or of every principal a DID is delegated to — the association mux exists to carry and the one most worth protecting. The map exists at the mediator whether or not this task does; the task makes it readable, and only to administrators (Consumer rule 3), never to a third party. Requiring a filter keeps a single request from dumping the mediator's whole delegation graph. Operators **SHOULD** treat a page as they treat the mediator's own connection logs. The mediator's identifier scope is public by design: it is the `audience` of every grant its members sign. Page sizes and timing reveal to an observer of the administrator's connection roughly how many grants a DID has.

### Retention

**Transient.** A page is superseded by the next read. A console **SHOULD** keep only its current view. The mediator writes no audit record of the read, as for [`messaging/mux/connections`](../../connections/0.1/spec.md); the request document, with its proof, is the evidence of who looked, should an operator keep it.

### Consent/purpose

The purpose is to operate the mediator: to find the delegations an administrative revocation or epoch advance will end, to answer a member DID's controller who reports a principal it does not recognise, and to diagnose why a grant does not attach. It is not for profiling which DIDs a principal holds, and a page **SHOULD NOT** be reused as a record of any person's associations.
