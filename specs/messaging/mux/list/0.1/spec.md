---
slug: messaging/mux/list
version: "0.1"
title: "Messaging — Mux List"
summary: "List mux delegations a page at a time: as a principal, the grants delegated to it and the attaches on this connection; as a member DID, the principals holding grants on it and its current epoch."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, list, delegation]
parties:
  - role: Requester (a principal, or a member DID or a principal holding a session attach of it)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: RECOMMENDED
  response: RECOMMENDED
  rationale: >-
    A read whose answer depends only on who asks, and the requester is authenticated by the connection it arrives on. A response proof lets a client that relies on the list after a reconnect — it is the source of truth for the member set — show where it came from.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replayed request discloses only what its requester was already entitled to see.
sideEffects:
  level: none
  rationale: >-
    Reads grant and attach records; changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses delegation metadata to one of its two parties: which member DIDs a principal holds grants from, or which principals hold grants on a member DID, with each grant's signed statement verbatim, its state, effective scope and expiry, and the member's epoch. Never message content.
retention:
  class: transient
  rationale: >-
    A read.
errorCodes:
  - code: messaging/mux:memberNotProven
    meaning: The request asks as `member` for a DID the requester neither is nor holds a live session attach of on the connection that carried the request.
    retryable: false
  - code: messaging/mux/list:cursorInvalid
    meaning: '`cursor` was not issued by this mediator for this requester and these request members.'
    retryable: false
related:
  - messaging/mux/grant
  - messaging/mux/attach
  - messaging/mux/update
  - messaging/mux/revoke
  - messaging/mux/revoke-all
---

## Abstract

The **Messaging — Mux List** Trust Task answers the question each side of a mux delegation needs answered after any disconnect: *who rides with whom?*

- **As `principal`**, the requester sees the grants delegated to it ([`messaging/mux/grant`](../../grant/0.1/spec.md)) — each grant's signed statement verbatim (member DID, grant identifier, scope, expiry, epoch, signing key and proof), with its state and effective scope and expiry (`active`, `suspended`, `staleKey`, `expired`, `epochRevoked`) — with `attachedNow` saying whether each rides the connection that carried the request, and, on the first page, the session attaches ([`messaging/mux/attach`](../../attach/0.1/spec.md)) on that connection.
- **As `member`**, the requester sees which principals hold grants on the member DID, in every state, and the member's current **epoch**, which any new grant must carry. This is how a DID's new owner discovers what was delegated before it took control.

Live events ([`messaging/mux/event`](../../event/0.1/spec.md)) describe one connection and are not stored; this list is the source of truth after a drop. A principal may hold thousands of grants, so the list is paged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.6 and §6).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled.
2. As `principal`: list the grants whose principal is the requester, narrowed to member DID `did` where present — active grants, and dead ones still within the retention window after their effective expiry — ordered by member DID then grant identifier. Set `attachedNow` on each. On the first page only, include in `attachments` the session attaches on the connection that carried the request; omit `attachments` when the request did not arrive on a mux connection.
3. As `member`: the member DID is `did`, or the requester when `did` is absent. Answer only when the member DID is the requester, or the requester holds a live session attach of it on the connection that carried the request; otherwise respond `messaging/mux:memberNotProven`. List the grants on that member DID — active, and dead ones within the retention window — ordered by principal DID then grant identifier, and include `epoch`, its current epoch. Omit `attachedNow` and `attachments`.
4. Return each grant's statement exactly as stored, proof included, so either party can re-verify it, with its state, its effective scope and expiry, and `narrowed: true` where messaging/mux/update narrowed them below what the statement signs.
5. Bind `nextCursor` to the requester and the other request members, and refuse any other cursor with `messaging/mux/list:cursorInvalid`. Omit `nextCursor` on the last page.
6. Never return a grant to anyone but its member, its principal, or an operator acting through the mediator's administrative tasks.

A **producer** that has just reconnected, or whose `ready` event reported `pendingNotices` above zero, **SHOULD** drain its notices and then list before it trusts its member set.

There is deliberately no single-grant `show` task: a grant is always reached through one of its two parties, both of whom list it with this task, and the tasks that act on one grant — [`messaging/mux/update`](../../update/0.1/spec.md) and [`messaging/mux/revoke`](../../revoke/0.1/spec.md) — already answer an unknown or foreign grant with a definite determination. Existence of a grant is never load-bearing for anyone else.

## Authorization

**Authority: being a party to the delegation.** A principal sees the grants that delegate to it, because it holds them; a member DID sees the grants on it, because they spend its authority. A member is proven by authenticating as it, or by a live session attach — itself proven by the member's own signature bound to this connection — on the connection the request arrives on. Whether a session attach should suffice, or only direct authentication, is an open question in the design; this version admits both. The transport and any document proof identify the requester ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the entitlement is the party relationship, read from the mediator's grant records.

## Definitions

- **`as`** — REQUIRED. [`MuxRole`](../../_shared/0.1/mux.schema.json#/$defs/MuxRole): `principal` or `member`.
- **`did`** — OPTIONAL. As `principal`, a member DID to narrow to; as `member`, the member DID to list (default: the requester).
- **`cursor`**, **`limit`** — OPTIONAL paging: an opaque token from `nextCursor`, and the most grants to return (1–500; default 100).
- **`grants`** (response) — [`GrantView`](../../_shared/0.1/mux.schema.json#/$defs/GrantView)s.
- **`attachments`** (response) — [`AttachView`](../../_shared/0.1/mux.schema.json#/$defs/AttachView)s, principal view, first page.
- **`epoch`** (response) — member view: the member DID's current epoch.
- **`nextCursor`** (response) — absent on the last page.

## Request

The requester sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A new owner lists the grants on its DID

```json
{
  "id": "urn:uuid:8b9c0d1e-2f3a-4b5c-8d6e-7f8a9b0c1d01",
  "type": "https://trusttasks.org/spec/messaging/mux/list/0.1",
  "issuer": "did:webvh:QmA1ice:alice.example:work",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T11:00:00Z",
  "payload": {
    "as": "member"
  }
}
```

## Response

The mediator responds with a page of grants, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### One principal, suspended by the new owner's key

```json
{
  "id": "urn:uuid:8b9c0d1e-2f3a-4b5c-8d6e-7f8a9b0c1d02",
  "type": "https://trusttasks.org/spec/messaging/mux/list/0.1#response",
  "threadId": "urn:uuid:8b9c0d1e-2f3a-4b5c-8d6e-7f8a9b0c1d01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T11:00:00Z",
  "payload": {
    "as": "member",
    "grants": [
      {
        "statement": {
          "type": "muxGrant",
          "issuer": "did:webvh:QmA1ice:alice.example:work",
          "audience": "did:web:mediator.example",
          "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
          "principal": "did:webvh:QmPrinc1pal:phone.example",
          "scope": [
            "receive",
            "pickup"
          ],
          "epoch": 3,
          "iat": 1794214800,
          "exp": 1796806800,
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
            "proofPurpose": "authentication",
            "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
          }
        },
        "state": "suspended",
        "scope": [
          "receive",
          "pickup"
        ],
        "expiresAt": "2026-12-09T09:00:00Z"
      }
    ],
    "epoch": 3
  }
}
```

The new owner authenticated with `#key-2`, which signed none of the DID's grants, so the mediator suspended the old principal's grant. Suspension leaves the epoch at 3; [`messaging/mux/revoke-all`](../../revoke-all/0.1/spec.md) with epoch 3 advances it and makes the hand-over final, so nothing signed under epoch 3 can be stored again.

## Security & Privacy

### Data carried

The response carries each grant's signed statement — member and principal DIDs, grant identifier, scope, validity window, epoch and the member's proof — with its state and effective scope and expiry, and the DIDs attached to this connection. A statement is signed, not secret, and goes only to a party to it. The request carries a role, perhaps a DID, and paging members. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The list is a map of which DIDs one party holds, and showing it to anyone but the two parties to each grant would publish that map. That is why the mediator answers a member view only to the member, or to a principal that just proved control of it, and why no third party can list. The member view does show a DID's new owner every principal its previous owner delegated to: that disclosure is the hand-over protection, and it is to the DID's current controller only. The mediator's identifier scope is public by design; it is the address every grant names. Page sizes and timing reveal roughly how many grants a requester holds.

### Retention

**Transient.** A client **SHOULD** treat the list as its current member set and replace, not accumulate, earlier copies.

### Consent/purpose

The purpose is for each side of a delegation to see, and so to manage, what it is party to: a principal reconciling its member set after a reconnect, a member DID — especially a new owner — discovering and ending delegations it did not make. Using the list to profile which DIDs one party holds is outside this task's purpose.
