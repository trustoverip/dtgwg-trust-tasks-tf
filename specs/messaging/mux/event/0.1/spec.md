---
slug: messaging/mux/event
version: "0.1"
title: "Messaging — Mux Event"
summary: "Live, unstored events from a mediator to the principal of one mux connection: ready (with the connection's nonce and limits), members attached, members detached with a reason, and inbound frames refused in-band."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, event, attach, detach, tsp-mux, websocket]
parties:
  - role: Mediator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Principal (the DID the connection is authenticated as)
    requirement: REQUIRED
    member: recipient
    identifierScope: any
proofRequirement:
  request: RECOMMENDED
  rationale: >-
    Events travel only over the principal's live connection, which already authenticates the mediator as sender (authcrypt for DIDComm, the TSP envelope's signature for TSP), and describe only that connection. A client that persists events or acts on one beyond the connection SHOULD require a proof; the durable record is messaging/mux/notice and messaging/mux/list, not this.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    An event has no response, so a principal cannot detect a delayed or replayed one by correlating a reply; issuedAt orders events in time, which matters when an attached and a detached event for the same member cross.
sideEffects:
  level: none
  rationale: >-
    The principal updates its view of the connection's member set; receiving an event changes nothing at the mediator.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Events carry connection metadata into the principal: which member DIDs joined or left and why, the connection's nonce and limits, and the sender, receiver and refusal code of a refused frame. Never a message body.
retention:
  class: transient
  rationale: >-
    An event describes the current connection only and is never stored by the mediator; the principal treats it as live state and reconciles with messaging/mux/list after a drop.
maxDocumentBytes:
  request: 1048576
  rationale: >-
    An attached or detached event batches up to 1000 members, each a DID and a few short members.
errorCodes: []
related:
  - messaging/mux/attach
  - messaging/mux/detach
  - messaging/mux/grant
  - messaging/mux/list
  - messaging/mux/notice
  - messaging/monitor/event
---

## Abstract

The **Messaging — Mux Event** Trust Task is how a mediator keeps the principal of a mux connection from ever having to *infer* the state of its connection. Four kinds of event exist:

- **`ready`** — the mediator's first frame on a mux connection after authentication. It carries the connection's `sessionId` and `muxNonce`, which session attaches sign over ([`messaging/mux/attach`](../../attach/0.1/spec.md)), the mediator's limits, and `pendingNotices`, the number of durable [`messaging/mux/notice`](../../notice/0.1/spec.md) documents waiting in the principal's inbox.
- **`attached`** — members joined the connection: through a stored grant on connect or when stored, through a session attach, or by reactivation.
- **`detached`** — members left the connection, each with a reason (revoked, epoch advanced, suspended under the unknown-key rule, key rotated, expired, blocked, superseded, an operator, or the principal's own detach). On connect, stored grants that fail re-validation are reported here too.
- **`refused`** — on a raw-TSP `tsp-mux` socket, an inbound frame was refused, reported in-band instead of by closing the socket.

Events are **live only**. The mediator never stores one, because each describes one connection; there is no response; and after a drop, [`messaging/mux/list`](../../list/0.1/spec.md) is the source of truth. Anything the principal must act on even when offline is also sent as a durable notice.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.2, §4.5, §4.9 and §4.10).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Members per kind

The payload is one object whose `event` member says which kind it is. Each kind carries exactly these members besides `event` (and optionally `ext`), and no others:

| `event` | REQUIRED | OPTIONAL |
|---|---|---|
| `ready` | `sessionId`, `muxNonce`, `maxDidsPerConnection`, `maxGrantLifetimeSeconds`, `pendingNotices` | `maxAttachLifetimeSeconds`, `sessionExpiresAt` |
| `attached` | `dids` — each entry `did`, `reason` (an [`AttachReason`](../../_shared/0.1/mux.schema.json#/$defs/AttachReason)), `scope`, `deliveryActive` | per entry: `grantId`, `expiresAt` |
| `detached` | `dids` — each entry `did`, `reason` (a [`DetachReason`](../../_shared/0.1/mux.schema.json#/$defs/DetachReason)) | per entry: `grantId` |
| `refused` | `frameId`, `code` | `sender`, `receiver`, `reason` |

A consumer **MUST** treat a document that carries a member not listed for its kind, or lacks a required one, as malformed and ignore it.

A conforming **producer** (the mediator) **MUST**:

1. Emit documents whose `type` is `https://trusttasks.org/spec/messaging/mux/event/0.1`, with itself as `issuer`, the principal as `recipient`, and an `issuedAt`, over the principal's live connection only — authcrypted to the principal for DIDComm, sealed to the principal's VID and sent verbatim for TSP. Every event is addressed to the principal, never to a member: the principal is the party authenticated on the connection, and an event about a member just detached needs none of that member's keys.
2. **Never store** an event in any inbox, and never resend one on another connection.
3. Send `ready` as the first frame on a mux connection after authentication, with a `muxNonce` of 32 fresh random bytes that belongs to this connection alone and is never reused. `pendingNotices` counts the messaging/mux/notice documents in the principal's inbox at that moment.
4. On connect, re-validate every stored grant of the principal, attach each valid one, and report them in **one** batched `attached` event with reason `connect` (split only where more than 1000 members change), followed by one `detached` event listing each grant that failed, with its reason.
5. Emit `attached` whenever a member joins, or a member's scope or expiry changes, and `detached` whenever a member leaves, every time, with its reason. A `detached` event for a member is sent only after the member's live traffic on this connection has stopped and any frame for it still queued has been dropped.
6. Set `deliveryActive: false` for a member whose own direct connection takes precedence, and report its later reactivation with an `attached` event, reason `reactivated`. A member losing live traffic to the DID's own direct connection, or to a newer connection of the same kind, is reported `detached` with reason `superseded`; it stays in the member set, inactive.

A conforming **consumer** (the principal) **MUST**:

1. Ignore an event whose `recipient` is not itself, or that arrives on a connection other than the one it describes.
2. Take `sessionId` and `muxNonce` for attach statements only from this connection's `ready` event.
3. When `pendingNotices` is above zero, drain its notices before it trusts its member set.
4. Surface each member's attach and detach, with the reason, to its application alongside the connection's own state. A client **MUST NOT** report a member as reachable without an `attached` event for it on the current connection, nor keep reporting it after a `detached` one.
5. Not respond. There is no `#response` form for this task.

### Raw-TSP sockets

A raw-TSP socket carries binary TSP frames only. Without mux, a mediator reports a refused inbound frame by closing the socket, which on a mux socket would take every member offline for one member's bad frame. The `tsp-mux` WebSocket subprotocol changes that:

1. **Negotiation.** The client offers `tsp`, `tsp-ack` and `tsp-mux`, in that order. A mediator selects `tsp-mux` only when it implements it and mux is enabled; an older mediator selects `tsp`, so a client never wrongly believes mux is on, and **MUST** treat a selected `tsp` or `tsp-ack` as "mux unavailable". `tsp-mux` implies `tsp-ack`.
2. **Control.** Mux Trust Tasks, and these events, travel as TSP Direct messages between the principal's VID and the mediator's on the same socket.
3. **Data frames are not tagged.** A TSP envelope names its receiver VID in the clear, and a member DID's TSP VID is the DID itself, so the client routes each delivered frame to its member by receiver.
4. **In-band refusals.** On a `tsp-mux` socket, a refused inbound frame does not close the socket. The mediator sends a `refused` event, where `frameId` is the lowercase hex SHA-256 of the frame's exact bytes — the identifier `tsp-ack` already uses — and `code` the stable refusal code a traffic-monitor `refused` event would report. The socket closes only for a failure of the principal itself: its authentication expiring, its account blocked, or a protocol violation.
5. **Inbox drain.** The mediator drains the principal's inbox and then, round-robin, each attached member's with `pickup`, in pages bounded per member. Acknowledgements are authorised against the owning member's `delete` scope; a member without `delete` has delete-on-send turned off, and its frames stay in its inbox.

How a DIDComm WebSocket becomes a mux connection is the transport's concern and is not defined here.

## Authorization

**Authority: none is exercised; the mediator informs the principal of the connection it authenticated.** Events are sent only to the principal of the connection they describe, and assert nothing the principal must accept: the principal's member set changes because of an attach, a grant, a revocation or an expiry, each authorised by its own task, and the event only reports it. On the consumer side, authenticity comes from the connection, which authenticates the mediator as sender, and optionally from the `proof`. Consumer rule 1 discards an event the principal did not ask for.

## Definitions

- **`event`** — REQUIRED. `ready`, `attached`, `detached` or `refused`.
- **`sessionId`**, **`muxNonce`** — this connection's identifier and nonce ([`SessionId`](../../_shared/0.1/mux.schema.json#/$defs/SessionId), [`MuxNonce`](../../_shared/0.1/mux.schema.json#/$defs/MuxNonce)).
- **`maxDidsPerConnection`** — the most members this connection may carry.
- **`maxGrantLifetimeSeconds`** — the longest `exp − iat` a grant may have.
- **`maxAttachLifetimeSeconds`**, **`sessionExpiresAt`** — the bounds on an attach statement's `exp`.
- **`pendingNotices`** — durable notices waiting in the principal's inbox.
- **`dids`** — `MemberChange` entries: `did`, `reason`, and for `attached` the `scope`, `deliveryActive` and, where it lapses, `expiresAt`; `grantId` where a grant is concerned.
- **`frameId`**, **`sender`**, **`receiver`**, **`code`**, **`reason`** — the refused frame's hash, its sender and receiver where readable, the refusal code, and a short explanation.

## Request

The mediator pushes events to the principal over its live connection. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Ready

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c01",
  "type": "https://trusttasks.org/spec/messaging/mux/event/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:01Z",
  "payload": {
    "event": "ready",
    "sessionId": "ses_7Hq2xV9mK3pL",
    "muxNonce": "Zk3xQ0b9Rw7mT2pN4vYc8LsH1eJ6aUdG5oWiKfB0nXq",
    "maxDidsPerConnection": 1000,
    "maxGrantLifetimeSeconds": 2592000,
    "maxAttachLifetimeSeconds": 86400,
    "sessionExpiresAt": "2026-11-09T21:00:01Z",
    "pendingNotices": 1
  }
}
```

### Grants attached on connect

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c02",
  "type": "https://trusttasks.org/spec/messaging/mux/event/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:01Z",
  "payload": {
    "event": "attached",
    "dids": [
      {
        "did": "did:webvh:QmA1ice:alice.example:work",
        "reason": "connect",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "scope": ["receive", "pickup"],
        "expiresAt": "2026-12-09T09:00:00Z",
        "deliveryActive": true
      }
    ]
  }
}
```

### A member suspended under the unknown-key rule

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c03",
  "type": "https://trusttasks.org/spec/messaging/mux/event/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T10:42:17Z",
  "payload": {
    "event": "detached",
    "dids": [
      {
        "did": "did:webvh:QmA1ice:alice.example:work",
        "reason": "suspendedUnknownKey",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c"
      }
    ]
  }
}
```

### A frame for one member refused on a tsp-mux socket

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c04",
  "type": "https://trusttasks.org/spec/messaging/mux/event/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:12:44Z",
  "payload": {
    "event": "refused",
    "frameId": "9f2c4e1a7b3d5f60812a4c6e8b0d2f4a6c8e0b2d4f6a8c0e2b4d6f8a0c2e4b6d",
    "sender": "did:webvh:QmA1ice:alice.example:work",
    "receiver": "did:web:bob.example",
    "code": "authorization.acl.denied",
    "reason": "bob's access list does not admit alice-work"
  }
}
```

## Security & Privacy

### Data carried

Events carry connection metadata: the connection's identifier and nonce, the mediator's limits, member DIDs with scopes, expiries and reasons, and for a refused frame its hash, sender, receiver and refusal code. The nonce is not a secret — an attach statement is useless without the member's signature — but it binds proofs to this connection and **MUST NOT** be reused. A producer **MUST NOT** put message content or key material in `reason` or in `ext`.

### Correlation

The events restate, to the principal, the member set the mediator already links to its connection; they add timing. A `refused` event names a member's counterparty, which the principal could read from the frame itself. The mediator's identifier scope is public by design: it is the `aud` of every proof the connection's members sign. An observer of the connection sees event sizes and timing, which roughly track member joins and leaves.

### Retention

**Transient.** The mediator stores no event. A principal **SHOULD** keep only its current view of the member set and replace it from messaging/mux/list after a drop.

### Consent/purpose

The purpose is to make each member's reachability on the connection observable, so that no client ever infers it. The events are not a log and are not for analytics.
