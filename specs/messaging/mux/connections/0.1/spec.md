---
slug: messaging/mux/connections
version: "0.1"
title: "Messaging — Mux Connections"
summary: "An administrator lists a mediator's live mux connections a page at a time — each connection's principal, wire and opening time, and the member DIDs it carries with their source, scope, expiry and whether their live delivery is active there."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, connections, admin, operator, console]
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
    The request is gated on administrative standing and returns which member DIDs each principal carries; a proof binds the read to the administrator's key independent of the transport, so the mediator can show who looked. The response is RECOMMENDED so an operator can keep a page as evidence of where a DID's traffic was going at a point in time.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on every administrative request; without issuedAt a captured request could be replayed indefinitely to watch which DIDs ride which connections.
sideEffects:
  level: none
  rationale: >-
    Reads the mediator's in-memory view of its live mux connections; changes nothing, persists nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Discloses connection metadata to an administrator: each live mux connection's session identifier, principal DID, wire and opening time, and the member DIDs it carries with their grant identifier or attach, scope, expiry and delivery state. Never message content, a signed statement, a nonce or key material. The request may carry a principal or member DID to filter on.
retention:
  class: transient
  rationale: >-
    A page describes live connections that change by the second; it is superseded by the next read.
maxDocumentBytes:
  response: 4194304
  rationale: >-
    A page carries at most 100 connections and 1000 member entries in total, each a DID of up to 2048 characters and a few short members.
errorCodes:
  - code: messaging/mux/connections:cursorInvalid
    meaning: '`cursor` was not issued by this mediator — or, where the answer covers one instance only, by this instance — for this requester and these request members, or it has expired.'
    retryable: false
related:
  - messaging/mux/event
  - messaging/mux/list
  - messaging/mux/grant
  - messaging/mux/attach
  - messaging/stats/show
---

## Abstract

The **Messaging — Mux Connections** Trust Task gives a mediator's operator what no other task does: a view of DID multiplexing as it is running. [`messaging/mux/list`](../../list/0.1/spec.md) shows each party to a delegation its own grants, and [`messaging/stats/show`](../../../stats/show/0.1/spec.md) counts websockets without telling a mux connection from any other. This task lists the **live mux connections** themselves — each one's principal, its wire (`didcomm` for a `didcomm-mux` socket, `tsp` for a `tsp-mux` socket), when it opened, and the member DIDs it carries, with how each came to be there (a stored grant, or a session attach), its scope and expiry, and whether its live delivery is active on that connection.

It answers the operator's two questions: *what is this principal carrying?* (filter by `principal`) and *where is this DID's traffic going?* (filter by `member`, which shows every connection carrying the DID, including those where it rides inactive because its own direct connection or a newer connection took its traffic — the `superseded` and `reactivated` transitions of [`messaging/mux/event`](../../event/0.1/spec.md)).

A connection may carry a thousand members or more, so the list is paged over connections, with a budget on member entries per page that splits a large connection across pages rather than truncating it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/mux/connections/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. To continue a listing it **MUST** send the previous page's `nextCursor` verbatim with the same `principal`, `member` and `memberLimit`.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled.
2. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
3. Respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
4. List the **live mux connections** — those it has sent a `ready` event on and that have not closed — ordered by `openedAt` then `sessionId`, narrowed to those authenticated as `principal` and to those carrying `member`, where present. A connection carrying no members is still listed, unless `member` is present.
5. Give each connection its exact `memberCount` — every member it carries, active or not — and in `members` a contiguous run of those members ordered by member DID, each with its `source`, `grantId` when and only when `source` is `grant`, the `scope` and `expiresAt` in force, and `deliveryActive`. With `member` present, `members` holds that member's entry alone.
6. Carry at most `memberLimit` member entries on the page, summed over its connections (default 1000). When the budget runs out inside a connection, end the page there; the next page begins with the same connection — same `sessionId`, `principal`, `wire` and `openedAt`, `memberCount` as it then stands — and its members after the last one returned. A page **MUST** carry at least one member entry when `memberLimit` is above zero and the page's first connection has members left to return, so that a listing always advances.
7. Set `coverage`. A mediator that runs as several instances and answers from shared state that includes every instance's live connections sets `allInstances`. One that can see only the connections held by the instance that answered **MUST** set `thisInstance`, and **MUST NOT** set `allInstances` unless it is a single instance. The answer **MUST NOT** imply completeness it does not have.
8. Bind `nextCursor` to the requester, the other request members and — under `thisInstance` — the instance that issued it, and refuse any other cursor, or one it has expired, with `messaging/mux/connections:cursorInvalid`. Omit `nextCursor` on the last page.
9. Never include a member's signed grant or attach statement, a connection's `muxNonce`, a message, or any key material.

The listing is a **live view, not a snapshot**: a cursor continues by position (`openedAt`, `sessionId`, member DID), so a connection that opens while an administrator pages appears at the end, one that closes is not returned again, and a split connection's later pages reflect its members as they then stand. A consumer **MUST NOT** restart a listing from the beginning when it is handed a cursor whose connection has since closed; it continues with the next connection in order.

An administrator's client that receives `thisInstance` **SHOULD** say so wherever it shows the result: an operator console that presents an instance's connections as the mediator's would make a member that is attached on another instance look unattached.

This task is a read-many with no `show` sibling. A connection's identity is its `sessionId`, which lasts only as long as the connection, and one principal's connections are reached with the `principal` filter; existence of a live connection is never load-bearing to an administrator in the way `notFound` would make it, since the next read may differ anyway ([CONTRIBUTING-SPECS "Read-one and read-many"](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)).

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin`, `rootAdmin`, or `mediator` role (Consumer rule 3), as for [`messaging/stats/show`](../../../stats/show/0.1/spec.md) and [`messaging/queue/list`](../../../queue/list/0.1/spec.md). The listing names principals and member DIDs other than the requester's, so there is no self-service form: a principal or a member DID reads its own delegations with [`messaging/mux/list`](../../list/0.1/spec.md).

Being a party to a delegation does not entitle anyone to this task, and administrative standing does not entitle an administrator to act on what it sees: ending a member's membership remains the business of the member, the principal, or an operator acting through the mediator's own administrative tasks. The `proof` establishes *who* asked and that the request is unaltered, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the mediator resolves the verified issuer to its account and decides from the role it finds there.

## Definitions

- **`principal`** — OPTIONAL. Only connections authenticated as this DID.
- **`member`** — OPTIONAL. Only connections carrying this member DID, active or not; each connection's `members` then holds that member alone.
- **`memberLimit`** — OPTIONAL. The page's budget of member entries across all its connections (0–1000; default 1000). `0` lists connections without their members.
- **`cursor`**, **`limit`** — OPTIONAL paging: an opaque token from `nextCursor`, and the most connections to return (1–100; default 50).
- **`connections`** (response) — `MuxConnection`s: `sessionId` ([`SessionId`](../../_shared/0.1/mux.schema.json#/$defs/SessionId)), `principal`, `wire` (`didcomm` or `tsp`), `openedAt`, `memberCount` and `members`.
- **`members`** — `ConnectionMember`s: `did`, `source` ([`MuxSource`](../../_shared/0.1/mux.schema.json#/$defs/MuxSource)), `grantId` ([`GrantId`](../../_shared/0.1/mux.schema.json#/$defs/GrantId), grant only), `scope` ([`MuxScope`](../../_shared/0.1/mux.schema.json#/$defs/MuxScope)), `expiresAt` and `deliveryActive` — false while the member's live traffic goes to its own direct connection or a newer connection, the state a `superseded` detach leaves it in.
- **`coverage`** (response) — `allInstances` or `thisInstance`: whether the answer covers every instance of the mediator, or only the one that answered.
- **`nextCursor`** (response) — absent on the last page.

The member budget, rather than a cap on members per connection, is what keeps a page bounded: a connection can carry `maxDidsPerConnection` members, which no schema bound limits, and a per-connection cap would leave the members past it unreachable or need a second cursor. Splitting a connection across pages keeps one cursor and every member reachable, and `memberCount` lets a client tell a split connection from a complete one.

## Request

The administrator sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Where is a DID's traffic going?

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e01",
  "type": "https://trusttasks.org/spec/messaging/mux/connections/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-11T09:30:00Z",
  "payload": {
    "member": "did:webvh:QmA1ice:alice.example:work"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-11T09:30:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Kq..."
  }
}
```

### A summary of every connection, without members

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e03",
  "type": "https://trusttasks.org/spec/messaging/mux/connections/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-11T09:31:00Z",
  "payload": {
    "memberLimit": 0,
    "limit": 100
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-11T09:31:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Wp..."
  }
}
```

## Response

The mediator responds with a page of connections, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) rather than a `#response` document.

### The DID rides two connections, active on one

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e02",
  "type": "https://trusttasks.org/spec/messaging/mux/connections/0.1#response",
  "threadId": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-11T09:30:00Z",
  "payload": {
    "connections": [
      {
        "sessionId": "ses_7Hq2xV9mK3pL",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "wire": "didcomm",
        "openedAt": "2026-11-11T07:58:12Z",
        "memberCount": 14,
        "members": [
          {
            "did": "did:webvh:QmA1ice:alice.example:work",
            "source": "grant",
            "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
            "scope": ["receive", "pickup"],
            "expiresAt": "2026-12-09T09:00:00Z",
            "deliveryActive": false
          }
        ]
      },
      {
        "sessionId": "ses_Q4nB8rT1wZ6y",
        "principal": "did:webvh:QmTab1et:tablet.example",
        "wire": "tsp",
        "openedAt": "2026-11-11T09:12:40Z",
        "memberCount": 2,
        "members": [
          {
            "did": "did:webvh:QmA1ice:alice.example:work",
            "source": "attach",
            "scope": ["receive", "pickup", "delete"],
            "expiresAt": "2026-11-11T21:12:40Z",
            "deliveryActive": true
          }
        ]
      }
    ],
    "coverage": "allInstances"
  }
}
```

The DID is carried by two principals. The tablet's newer `tsp-mux` connection took its live traffic by session attach, so the phone's connection, which carries it through a stored grant, holds it inactive.

### Refused: not an administrator

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-11T09:30:00Z",
  "payload": {
    "code": "permissionDenied",
    "message": "messaging/mux/connections requires an administrator account.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The response carries, per live mux connection, its session identifier, principal DID, wire and opening time, and per member its DID, source, grant identifier, scope, expiry and delivery state. It carries no signed statement — those stay with [`messaging/mux/list`](../../list/0.1/spec.md), which shows them to their parties — no connection nonce, no message content, no message count and no key material. The session identifier is not a credential: it names a connection without granting anything on it. The request carries at most a principal DID, a member DID and paging members. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

This task exposes the one thing mux makes linkable: which DIDs travel together. A principal carrying several member DIDs links them, to whoever can see its connection, as one person's or one device's — and a `member` filter shows every principal a DID is delegated to, with when each connected. That map exists at the mediator whether or not this task does; the task makes it readable, and only to administrators (Consumer rule 3), never to a principal or member about anyone but themselves. Operators **SHOULD** treat a page as they treat the mediator's own connection logs. The mediator's identifier scope is public by design: it is the `audience` of every grant and attach its members sign. Page sizes and timing reveal to an observer of the administrator's connection roughly how much mux traffic the mediator carries.

### Retention

**Transient.** A page describes connections that may close a second later, and is superseded by the next read. A console **SHOULD** keep only its current view and replace it, rather than accumulate pages into a history of who carried whom. The mediator writes no audit record of the read, as for [`messaging/stats/show`](../../../stats/show/0.1/spec.md) and [`messaging/queue/list`](../../../queue/list/0.1/spec.md); the request document, with its proof, is the evidence of who looked, should an operator keep it.

### Consent/purpose

The purpose is to operate the mediator: to see whether mux connections are carrying the members they should, to find where a DID's live traffic is going when its controller reports it unreachable, and to size `maxDidsPerConnection`. It is not for profiling which DIDs a principal holds, and a page **SHOULD NOT** be reused as a record of any person's associations.
