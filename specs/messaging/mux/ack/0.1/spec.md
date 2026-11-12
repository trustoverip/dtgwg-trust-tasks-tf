---
slug: messaging/mux/ack
version: "0.1"
title: "Messaging — Mux Ack"
summary: "A principal acknowledges frames delivered on its mux connection; the mediator deletes each one the principal, or the member it was for, may delete."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, acknowledge, delete, tsp, websocket]
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
    An ack deletes only frames the mediator already delivered on the connection the request arrives on, which authenticates the principal, and only where the principal or the member the frame was for holds the right to delete it. It grants nothing and reaches no other connection, so the connection's authentication is the authority; a proof adds evidence but no permission.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task deletes stored frames, which makes it consequential (SPEC §7.3 item 17); issuedAt bounds the window in which a duplicate is absorbed.
sideEffects:
  level: mutating
  rationale: >-
    Deletes the mediator's stored copy of frames the principal says it has taken from the connection. The content is with the principal, which received it; what goes is the mediator's copy and the redelivery it would have caused. Acknowledging an id again, or one already gone, changes nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names message ids the mediator itself handed to the connection; the response sorts the same ids. Nothing is learnt that the mediator did not already hold.
retention:
  class: transient
  rationale: >-
    The mediator acts and keeps nothing of the request beyond the deletions it makes.
errorCodes:
  - code: messaging/mux:notMuxConnection
    meaning: The request did not arrive on a mux connection, so there are no members' frames it could acknowledge. Send it on the connection the frames were delivered on.
    retryable: false
related:
  - messaging/mux/event
  - messaging/mux/attach
  - messaging/mux/grant
  - messaging/mux/notice
---

## Abstract

The **Messaging — Mux Ack** Trust Task acknowledges frames a mediator delivered on a mux connection, so the mediator deletes its stored copies. It is sent on the connection the frames came on, which is the only place that knows which members the connection carries and with what scope.

A connection whose delivery is delete-to-ack keeps every frame it sends until the client says it has taken it; on a raw-TSP `tsp-mux` connection (see [`messaging/mux/event`](../../event/0.1/spec.md), "Raw-TSP sockets") that is every frame. A frame for the principal is the principal's to delete. A frame for a member may be deleted only while that member is on the connection with the `delete` scope. This task applies both rules on the connection itself, so it gives the same answer whichever instance of a clustered mediator would otherwise have taken an out-of-band delete.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.10).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the principal) **MUST** send the request on the mux connection the frames were delivered on, naming each id at most once, and **MUST** acknowledge a frame only once it has handled it: an acknowledged frame is not delivered again.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `messaging/mux:notMuxConnection` when the request did not arrive on a mux connection.
2. For each id: delete the stored message when it is in the principal's inbox, or in the inbox of a member that is on this connection, at the time of the request, with the `delete` scope. List it in `acked` only once it is deleted.
3. List every other id in `refused`: with `notPermitted` when the message is held for a member this connection carries and the member lacks `delete` (or its account no longer stands), and with `notFound` otherwise. A message held for a DID the connection does not carry **MUST** be answered `notFound`, as if it did not exist: an id is a hash of the message, so telling the two apart would let a principal confirm another DID's mail. A refused message is left as it was.
4. Answer on the connection the request arrived on. Because acknowledging is idempotent, the consumer **MAY** deliver the response on the connection only, without storing it; a response it does store is acknowledged like any other frame.

An id is the lowercase hexadecimal SHA-256 of the stored message: for a TSP message, of the base64url (no padding) encoding of its qb2 bytes, as for `tsp-ack`; for a DIDComm message, of the message as stored, as a message-pickup delete names it.

## Authorization

**Authority: being the principal of the connection the frames were delivered on, and, for a member's frame, the member's `delete` scope on that connection.** The connection authenticates the principal; the member's own grant or attach, already verified when it joined the connection, carries its consent to the principal deleting its messages. A member that has left the connection, or never gave `delete`, keeps its messages: they stay in its inbox for its next connection. The task reaches no other connection, and no message the mediator did not hold for the principal or a current member.

## Definitions

- **`messageIds`** — REQUIRED. 1–100 ids of frames delivered on this connection, each the lowercase hexadecimal SHA-256 of the stored message.
- **`acked`** (response) — ids whose messages were deleted.
- **`refused`** (response) — ids that were not, each with its `reason`: `notFound` or `notPermitted`.

## Request

The principal sends the request to the mediator on the mux connection. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Acknowledge two frames

```json
{
  "id": "urn:uuid:3c1d5e2f-8a4b-4c6d-9e0f-1a2b3c4d5e01",
  "type": "https://trusttasks.org/spec/messaging/mux/ack/0.1",
  "issuer": "did:webvh:QmPrinc1pal:phone.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-12T09:30:00Z",
  "payload": {
    "messageIds": [
      "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
      "60303ae22b998861bce3b28f33eec1be758a213c86c93c076dbe9f558c11c752"
    ]
  }
}
```

## Response

The mediator responds after the deletions, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### One deleted, one a member's without `delete`

```json
{
  "id": "urn:uuid:3c1d5e2f-8a4b-4c6d-9e0f-1a2b3c4d5e02",
  "type": "https://trusttasks.org/spec/messaging/mux/ack/0.1#response",
  "threadId": "urn:uuid:3c1d5e2f-8a4b-4c6d-9e0f-1a2b3c4d5e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-12T09:30:00Z",
  "payload": {
    "acked": ["9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"],
    "refused": [
      {
        "messageId": "60303ae22b998861bce3b28f33eec1be758a213c86c93c076dbe9f558c11c752",
        "reason": "notPermitted"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Message ids in, the same ids sorted into two lists out. An id is a hash of a message the mediator stores and already delivered on this connection, so it carries nothing the parties did not hold. Because an id is a content hash, a principal could name one it guessed; consumer rule 3 keeps the answer from saying whether anyone else holds such a message. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request tells the mediator when the principal handled each frame, which it would learn from an out-of-band delete as well. It names only frames of this connection, so it links nothing the connection did not already link. The mediator's identifier scope is public by design: it is the address every principal, member and peer already uses to reach it, and the `aud` every mux proof names.

### Retention

Nothing of the request is kept. The deletions are the effect; a mediator's audit log, if its operator keeps one, records them as it records any delete.

### Consent/purpose

The purpose is delivery bookkeeping: releasing the mediator's copy of what the principal has taken. A member's consent is the `delete` scope it gave the principal; without it the member's messages stay.
