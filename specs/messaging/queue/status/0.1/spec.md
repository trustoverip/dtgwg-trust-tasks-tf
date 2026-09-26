---
slug: messaging/queue/status
version: "0.1"
title: "Messaging — Show Queue Status"
summary: "An account controller, or an administrator, reads one account's receive and send queues — depth, size, age, limits, and optionally which counterparties account for them."
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
    Whether the read is permitted depends on who asks — the account's own controller, or an administrator naming
    another account. A proof binds the request to the requester's key independent of the transport, so the mediator
    decides from a verified identity. The response is RECOMMENDED so a controller can retain it as evidence of its
    queue state when disputing a refusal.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on these requests; without issuedAt a captured request could be replayed
    indefinitely to watch an account's queues.
sideEffects:
  level: none
  rationale: >-
    A read of queue state the mediator already tracks; it persists nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns one account's queue depths, sizes, ages, and limits, and — when requested — the counterparties that
    account for them, with per-counterparty counts. That reveals who the account exchanges messages with, but no
    message content.
retention:
  class: transient
  rationale: >-
    A status reading is superseded by the next one and has no value once the back-pressure it was read to diagnose
    is resolved.
errorCodes:
  - code: messaging/queue/status:unknownAccount
    meaning: The named account does not exist at this mediator.
    retryable: false
related:
  - messaging/queue/list
  - messaging/queue/purge
  - messaging/message/list
  - messaging/account/get
---

## Abstract

The **Messaging — Show Queue Status** Trust Task returns both queues of one account: how many messages each holds, their stored size, the age of the oldest, the effective limit and saturation, and how many have been delivered but not yet deleted. With `includePeers` it also breaks each queue down by counterparty.

That breakdown is the view that diagnoses most queue incidents. A message is held against its **sender's** send queue until its **recipient** deletes it, so a recipient that stops acknowledging — offline, crashing on one message, or failing its deletes — fills the send queue of every peer talking to it, and those peers then hit their send limit and stop being able to send anything. The peer whose queue is full did nothing wrong; `sendPeers` names the recipient that is actually stalled.

The account's own controller may read its own queues; an administrator may read any account's. This task is the **read-one** half of the split pair whose read-many half is [`messaging/queue/list`](../../list/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/queue/status/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. It omits `did` to read its own account.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
2. Resolve the target account: `payload.did` when present, otherwise the requester's own account.
3. Respond with `permissionDenied` where the target is not the requester's own account and the requester's account type is not `admin`, `rootAdmin`, or `mediator`.
4. Respond with `messaging/queue/status:unknownAccount` where the target account does not exist. For a requester without administrative standing naming another account, rule 3 applies first, so the error never reveals whether that account exists.
5. Return the account's [`QueueSummary`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/QueueSummary) with each queue's `limit` as the **effective** limit — the account override if set, otherwise the mediator default.
6. When `includePeers` is present, return up to that many counterparties per queue, ordered by message count descending, as [`PeerDepth`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/PeerDepth) entries: `receivePeers` by sender, `sendPeers` by recipient. Omit both members when `includePeers` is absent.

## Authorization

The entitlement is **control of the account, or administrative standing**. The account's own controller — the verified issuer resolving to the target account — may read its queues; any other requester must hold the `admin`, `rootAdmin`, or `mediator` role (Consumer rules 2–3). A self-read needs no capability beyond having an account: it discloses nothing the controller could not already observe by fetching its own messages.

The `proof` establishes *who* asked; the decision is the comparison between that identity and the target account, then the requester's role ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- `did` — the target account ([`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid)). Omitted = the requester's own account.
- `includePeers` — also return the top N counterparties of each queue.
- `queues` — the account's [`QueueSummary`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/QueueSummary).
- `receivePeers` — the senders whose messages fill the receive queue.
- `sendPeers` — the recipients that have not yet deleted this account's sent messages. In a send queue, `deliveredUnacked` close to `count` for one peer means that peer is receiving but not acknowledging.

## Request

The requester names an account, or omits it for its own; see the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator diagnosing a full send queue

```json
{
  "id": "urn:uuid:9a4c1e7b-2d5f-4a80-b3c6-8e1f0d2a3b01",
  "type": "https://trusttasks.org/spec/messaging/queue/status/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:07:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "includePeers": 5
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:07:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Tr..."
  }
}
```

## Response

The mediator responds with `https://trusttasks.org/spec/messaging/queue/status/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### One stalled recipient fills Alice's send queue

```json
{
  "id": "urn:uuid:9a4c1e7b-2d5f-4a80-b3c6-8e1f0d2a3b02",
  "type": "https://trusttasks.org/spec/messaging/queue/status/0.1#response",
  "threadId": "urn:uuid:9a4c1e7b-2d5f-4a80-b3c6-8e1f0d2a3b01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:07:00Z",
  "payload": {
    "queues": {
      "did": "did:web:alice.example",
      "accountType": "standard",
      "receive": { "count": 2, "bytes": 4810, "limit": 1000, "saturation": 0.002, "oldestAgeSeconds": 40 },
      "send": { "count": 931, "bytes": 1804412, "limit": 1000, "saturation": 0.931, "oldestAgeSeconds": 86112, "deliveredUnacked": 931 }
    },
    "receivePeers": [
      { "peer": "did:web:bob.example", "count": 2, "bytes": 4810, "oldestAgeSeconds": 40 }
    ],
    "sendPeers": [
      { "peer": "did:web:carol.example", "count": 927, "bytes": 1796001, "oldestAgeSeconds": 86112 },
      { "peer": "did:web:bob.example", "count": 4, "bytes": 8411, "oldestAgeSeconds": 12 }
    ]
  }
}
```

927 of Alice's 931 held messages are waiting on Carol, the oldest for a day, and all have been delivered: Carol is receiving and not deleting. The fix is at Carol, not Alice.

## Security & Privacy

### Data carried

The request carries an optional account identifier and a count. The response carries that account's queue depths and, when requested, the identifiers of its counterparties with per-counterparty counts and sizes. `sendPeers` and `receivePeers` are the sensitive members: they are a partial map of who the account talks to. A producer that does not need them **SHOULD** omit `includePeers`. No message content is carried, and a mediator **MUST NOT** place any in `ext`.

### Correlation

Counterparty identifiers are the mediator's account identifiers and join directly to every other `messaging/*` read. An administrator reading successive statuses can observe an account's relationships over time; this is inherent to diagnosing back-pressure and is available only to the account itself or an administrator.

### Retention

Transient. The reading is superseded by the next; neither side needs to keep it once the diagnosis is made. The mediator need not retain the request beyond its replay window.

### Consent/purpose

The data is collected to diagnose and relieve queue back-pressure. The counterparty breakdown in particular is for finding a stalled peer, and should not be reused to profile an account's relationships.
