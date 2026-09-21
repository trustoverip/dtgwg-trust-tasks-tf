---
slug: messaging/message/list
version: "0.1"
title: "Messaging — List Messages"
summary: "An account controller, or an administrator, pages through the metadata of the messages in one account queue — sender, recipient, size, age, protocol, and delivery state — never their content."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Account controller or administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Whether the read is permitted depends on who asks — the account's controller, or an administrator reading another
    account's traffic metadata. A proof binds the request to the requester's key independent of the transport. The
    response is RECOMMENDED so a requester can retain a listing as evidence of what was queued at a point in time.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on these requests; without issuedAt a captured request could be replayed
    indefinitely to watch an account's traffic.
sideEffects:
  level: none
  rationale: >-
    A read of stored metadata. Listing does not deliver, mark, or delete any message.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns per-message metadata — identifiers, counterparties, sizes, timestamps, protocol, and delivery state.
    Never the message body.
retention:
  class: transient
  rationale: >-
    A listing is a view of a queue that changes continuously; it has no value once the requester has acted on it.
errorCodes:
  - code: messaging/message/list:unknownAccount
    meaning: The named account does not exist at this mediator.
    retryable: false
related:
  - messaging/message/get
  - messaging/message/delete
  - messaging/queue/status
  - messaging/queue/purge
---

## Abstract

The **Messaging — List Messages** Trust Task pages through one of an account's queues, oldest first, returning each message's [`MessageMeta`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MessageMeta): its identifier, sender and recipient where known, size, when it was stored and will expire, the protocol it travelled in, and whether it has been delivered and is awaiting acknowledgement. It never returns a message body. An optional `peer` narrows the listing to messages exchanged with one counterparty — in a send queue, the messages waiting on one recipient.

This task is the **read-many** half of a split pair ([CONTRIBUTING-SPECS "Read-one and read-many"](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)). Its read-one sibling, [`messaging/message/get`](../../get/0.1/spec.md), fetches one message by `msgId` with a definite not-found and a stricter authorization, because it returns the stored message itself.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/message/list/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. To continue a listing it **MUST** send the prior page's `nextCursor` verbatim with the same `did`, `queue`, and `peer`.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
2. Resolve the target account: `payload.did` when present, otherwise the requester's own account.
3. Where the target is the requester's own account, respond with `permissionDenied` unless the account holds the `local` capability.
4. Where the target is another account, respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
5. Respond with `messaging/message/list:unknownAccount` where the target account does not exist. Rules 3–4 apply first.
6. Return the messages of the named `queue`, narrowed to `peer` when present, **oldest first**, at most `limit` per page, each as a [`MessageMeta`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MessageMeta).
7. **NOT** include any message body, or any part of one, in the response or in `ext`.
8. Have no side effect on the listed messages: listing **MUST NOT** change a message's delivery state, extend or shorten its expiry, or count as delivery.

A cursor continues forward from the last message returned. Messages removed between pages are simply absent; messages arriving between pages appear at the end. A listing is therefore not a snapshot, and a producer that needs a consistent count reads [`messaging/queue/status`](../../../queue/status/0.1/spec.md).

## Authorization

The entitlement is **control of the account, or administrative standing**. The account's own controller may list its own queues if the account holds the `local` capability — the capability that makes the mediator store its messages at all (Consumer rule 3). Any other requester must hold the `admin`, `rootAdmin`, or `mediator` role (rule 4). An administrator sees metadata only; reading a stored message itself is [`messaging/message/get`](../../get/0.1/spec.md), which gates another account's messages more strictly.

The `proof` establishes *who* asked; the decision is the comparison between that identity, the target account, and the requester's role ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- `did` — the target account ([`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid)). Omitted = the requester's own account.
- `queue` — the [`Queue`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/Queue) to list.
- `peer` — only messages exchanged with this counterparty: in a send queue, the recipient; in a receive queue, the sender.
- `cursor`, `limit` — opaque continuation and page size.
- `messages` — the page, oldest first.

## Request

The requester names the queue and optionally the account and counterparty; see the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice lists the messages still waiting on Carol

```json
{
  "id": "urn:uuid:e5b2d9c0-8a1f-4e37-a6d4-1c9f3b7e2a01",
  "type": "https://trusttasks.org/spec/messaging/message/list/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:12:00Z",
  "payload": {
    "queue": "send",
    "peer": "did:web:carol.example",
    "limit": 2
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-09-21T10:12:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Hn..."
  }
}
```

## Response

The mediator responds with `https://trusttasks.org/spec/messaging/message/list/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### Two delivered, unacknowledged messages

```json
{
  "id": "urn:uuid:e5b2d9c0-8a1f-4e37-a6d4-1c9f3b7e2a02",
  "type": "https://trusttasks.org/spec/messaging/message/list/0.1#response",
  "threadId": "urn:uuid:e5b2d9c0-8a1f-4e37-a6d4-1c9f3b7e2a01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-09-21T10:12:00Z",
  "payload": {
    "messages": [
      {
        "msgId": "1758362400123-0",
        "queue": "send",
        "size": 1931,
        "receivedAt": "2026-09-20T10:05:12Z",
        "expiresAt": "2026-09-27T10:05:12Z",
        "from": "did:web:alice.example",
        "to": "did:web:carol.example",
        "protocol": "didcomm",
        "deliveryState": "delivered",
        "deliveredAt": "2026-09-20T10:05:13Z"
      },
      {
        "msgId": "1758362417840-0",
        "queue": "send",
        "size": 1944,
        "receivedAt": "2026-09-20T10:05:29Z",
        "expiresAt": "2026-09-27T10:05:29Z",
        "from": "did:web:alice.example",
        "to": "did:web:carol.example",
        "protocol": "tsp",
        "deliveryState": "delivered",
        "deliveredAt": "2026-09-20T10:05:30Z"
      }
    ],
    "nextCursor": "MTc1ODM2MjQxNzg0MC0w"
  }
}
```

Both messages reached Carol within a second of being stored and have sat undeleted for a day since.

## Security & Privacy

### Data carried

The request carries an account identifier, a queue, and optionally a counterparty. The response carries per-message metadata: identifiers, counterparties, sizes, timestamps, protocol, and delivery state. That is traffic metadata — who exchanged messages with whom, when, and how much — and is sensitive even though no content is carried. The mediator **MUST NOT** return any part of a message body, and **MUST NOT** place content in `ext`.

### Correlation

`msgId` values join a listing to [`messaging/message/get`](../../get/0.1/spec.md) and [`messaging/message/delete`](../../delete/0.1/spec.md), and to the traffic monitor's events, by design. `from` and `to` join to every other `messaging/*` read. Timing and sizes can correlate a message across mediators for an observer with visibility of several; that is inherent to relaying and not something this task adds.

### Retention

Transient. The listing reflects a queue that changes continuously and has no value once the requester has acted on it. The mediator need not retain the request beyond its replay window.

### Consent/purpose

The data is collected so an account controller can see and manage its own queue, and so an administrator can diagnose a queue that is not draining. An administrator should not reuse listings as a record of an account's communications.
