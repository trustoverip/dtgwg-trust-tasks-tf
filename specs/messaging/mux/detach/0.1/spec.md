---
slug: messaging/mux/detach
version: "0.1"
title: "Messaging — Mux Detach"
summary: "A principal removes session attaches from its mux connection; the members' live traffic stops before the mediator answers. Stored grants are untouched."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, detach, session, websocket]
parties:
  - role: Principal (the DID the connection is authenticated as)
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
    A detach only removes what the same principal attached on the same connection, and the connection authenticates the principal. It narrows what the principal receives and grants nothing, so a proofless detach can do no more harm than closing the connection would.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task mutates the connection's member set, which makes it consequential (SPEC §7.3 item 17); issuedAt keeps a delayed copy from removing an attach made after it.
sideEffects:
  level: mutating
  rationale: >-
    Removes session attaches from one connection. Reversible by attaching again with a fresh proof; nothing stored changes and no message is lost — frames still queued for a detached member stay in its inbox.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names member DIDs already attached to the principal's connection; the response says which were detached. It reveals nothing the principal did not already hold.
retention:
  class: transient
  rationale: >-
    The mediator acts and keeps nothing of the request.
errorCodes:
  - code: messaging/mux:notMuxConnection
    meaning: The request did not arrive on a mux connection, so there is no session attach it could remove. Send it on the connection that holds the attaches.
    retryable: false
related:
  - messaging/mux/attach
  - messaging/mux/revoke
  - messaging/mux/list
  - messaging/mux/event
---

## Abstract

The **Messaging — Mux Detach** Trust Task removes session attaches, made with [`messaging/mux/attach`](../../attach/0.1/spec.md), from the connection that carries the request. The mediator stops live delivery for each detached member **before** it answers, drops any frame for that member still queued on the connection — the message itself stays in the member's inbox — and emits a `detached` [`messaging/mux/event`](../../event/0.1/spec.md).

Detach touches session attaches only. A member that is also on the connection through a stored grant stays, under the grant's scope; removing it for good is [`messaging/mux/revoke`](../../revoke/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.8 and §7.3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the principal) **MUST** send the request on the mux connection holding the attaches, naming each DID at most once.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `messaging/mux:notMuxConnection` when the request did not arrive on a mux connection.
2. For each DID with a session attach on this connection: remove the attach and wait until the connection's streaming path confirms it carries no more live traffic for that member; drop, rather than send, any frame for that member already queued on the connection. Only then respond. A frame for a detached member **MUST NOT** leave the connection after the response.
3. List each such DID in `detached`. Where the DID is also a member through a stored grant, list it in `retainedByGrant` too: it stays on the connection under the grant's scope.
4. List every other requested DID in `notAttached`. Detaching what is not attached is not an error: the requested state already holds.
5. Emit one batched `detached` [`messaging/mux/event`](../../event/0.1/spec.md) with reason `detachRequested` for the DIDs that left the connection. A retained DID whose effective scope changed is reported by an `attached` event with its new scope.

## Authorization

**Authority: being the principal of the connection that holds the attach.** A session attach exists only on the connection it was made on, for the principal that made it, so the requester's standing is established by the connection the request arrives on. No member is consulted: a principal may always decline to carry traffic. This task cannot reach another connection's attaches or any stored grant.

## Definitions

- **`dids`** — REQUIRED. 1–256 member DIDs ([`MuxDid`](../../_shared/0.1/mux.schema.json#/$defs/MuxDid)).
- **`detached`** (response) — DIDs whose session attach was removed.
- **`retainedByGrant`** (response) — the subset of `detached` that remain members through a stored grant.
- **`notAttached`** (response) — DIDs that had no session attach here.

## Request

The principal sends the request to the mediator on the mux connection. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Detach two members

```json
{
  "id": "urn:uuid:7a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c01",
  "type": "https://trusttasks.org/spec/messaging/mux/detach/0.1",
  "issuer": "did:webvh:QmPrinc1pal:phone.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T09:30:00Z",
  "payload": {
    "dids": [
      "did:webvh:QmA1ice:alice.example:work",
      "did:peer:2.Ez6LSexamplePeerDid"
    ]
  }
}
```

## Response

The mediator responds after the detaches have taken effect, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### One detached, one never attached

```json
{
  "id": "urn:uuid:7a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c02",
  "type": "https://trusttasks.org/spec/messaging/mux/detach/0.1#response",
  "threadId": "urn:uuid:7a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:30:00Z",
  "payload": {
    "detached": ["did:webvh:QmA1ice:alice.example:work"],
    "retainedByGrant": [],
    "notAttached": ["did:peer:2.Ez6LSexamplePeerDid"]
  }
}
```

## Security & Privacy

### Data carried

Member DIDs in, the same DIDs sorted into three lists out. Nothing else. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request names DIDs the mediator already knows ride this connection, so it adds no link the attach did not already reveal. Its timing tells the mediator when the principal stopped carrying a member. The mediator's identifier scope is public by design: it is the address every principal, member and peer already uses to reach it, and the `aud` every mux proof names.

### Retention

Nothing is kept. Once the attach is gone, the mediator holds nothing linking the member to this connection, beyond whatever audit log its operator keeps.

### Consent/purpose

The purpose is to end connection sharing for a member, at the principal's choice. It is the narrowing counterpart of attach and needs nothing from the member.
