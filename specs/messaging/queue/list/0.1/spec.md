---
slug: messaging/queue/list
version: "0.1"
title: "Messaging — List Queues"
summary: "An administrator ranks a mediator's accounts by the depth, size, age, or saturation of one of their queues, to find the accounts whose queues are backing up."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request is gated on administrative standing and returns per-account activity; a proof binds the read to the
    administrator's key independent of the transport so the mediator can show who enumerated its accounts. The
    response is RECOMMENDED so an operator can retain a ranking as evidence of an incident's state.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on every administrative request; without issuedAt a captured request
    could be replayed indefinitely to re-read account activity.
sideEffects:
  level: none
  rationale: >-
    A read of queue depths the mediator already tracks, or of its latest periodic survey; it persists nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns per-account queue depths, sizes, ages, limits, and roles. That reveals which accounts are active and how
    much traffic they hold, but no message content and no counterparty.
retention:
  class: transient
  rationale: >-
    A ranking is a snapshot superseded by the next survey; it has no value once the incident it was read for is
    resolved.
errorCodes: []
related:
  - messaging/queue/status
  - messaging/queue/purge
  - messaging/stats/show
  - messaging/account/list
---

## Abstract

The **Messaging — List Queues** Trust Task returns a mediator's accounts ranked, in descending order, by one property of one of their queues: message `count`, stored `bytes`, the age of the `oldest` message, or `saturation` against the queue's limit. It answers the first question of a queue incident — *which accounts are backing up?* — without the administrator paging every account through [`messaging/account/list`](../../../account/list/0.1/spec.md) and sorting client-side.

This task is the **read-many** half of a split pair ([CONTRIBUTING-SPECS "Read-one and read-many"](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)). Its read-one sibling is [`messaging/queue/status`](../../status/0.1/spec.md), which returns one named account's queues with a definite `unknownAccount` when the account does not exist, a per-counterparty breakdown, and self-service access. A ranking has no per-record identity to fail on, so the two are not collapsed into one task.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/queue/list/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. To continue a ranking it **MUST** send the prior page's `nextCursor` verbatim with the same `queue`, `sort`, and `minCount`.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
2. Respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
3. Rank by the requested `queue` (default `receive`) and `sort` key (default `count`), descending, omitting accounts whose ranked queue holds fewer than `minCount` messages (default 1). Accounts ranked by `oldest` or `saturation` for which the value is unknown or undefined (an empty queue, an unlimited limit) rank last.
4. Return each account as a [`QueueSummary`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/QueueSummary) carrying **both** its queues, so the administrator sees the other queue without a second read.
5. Set `snapshotAt` to when the ranking was computed. A mediator **MAY** serve the ranking from a periodic survey rather than a live scan; where it does, `snapshotAt` is the survey time, and `truncated` is `true` if the survey stopped before covering every account.
6. Keep one ranking stable across its pages: a cursor **MUST** continue the snapshot the first page was drawn from, or fail with `malformedRequest` if that snapshot is no longer available — never silently restart on a newer one.

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin`, `rootAdmin`, or `mediator` role (Consumer rule 2). The ranking enumerates accounts other than the requester's, so there is no self-service form; an account controller reads its own queues with [`messaging/queue/status`](../../status/0.1/spec.md).

The `proof` establishes *who* asked and that the request is unaltered, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- `queue` — the [`Queue`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/Queue) whose property drives the ranking.
- `sort` — `count` (messages), `bytes` (stored size), `oldest` (age of the oldest message), or `saturation` (count ÷ effective limit).
- `minCount` — skip accounts with fewer messages than this in the ranked queue.
- `cursor`, `limit` — opaque continuation and page size.
- `queues` — the page, each entry a [`QueueSummary`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/QueueSummary).
- `snapshotAt`, `truncated` — when the ranking was computed, and whether it is incomplete.

## Request

The administrator sends the ranking parameters to the mediator; see the top-level schema in [`payload.schema.json`](payload.schema.json).

### The ten send queues closest to their limits

```json
{
  "id": "urn:uuid:7d2e5a90-1c3b-4f6e-8a2d-4b9c0e1f2a01",
  "type": "https://trusttasks.org/spec/messaging/queue/list/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:05:00Z",
  "payload": {
    "queue": "send",
    "sort": "saturation",
    "limit": 10
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z2Lm..."
  }
}
```

## Response

The mediator responds with `https://trusttasks.org/spec/messaging/queue/list/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### One account near its send limit

```json
{
  "id": "urn:uuid:7d2e5a90-1c3b-4f6e-8a2d-4b9c0e1f2a02",
  "type": "https://trusttasks.org/spec/messaging/queue/list/0.1#response",
  "threadId": "urn:uuid:7d2e5a90-1c3b-4f6e-8a2d-4b9c0e1f2a01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:05:00Z",
  "payload": {
    "queues": [
      {
        "did": "did:web:alice.example",
        "accountType": "standard",
        "receive": { "count": 2, "bytes": 4810, "limit": 1000, "saturation": 0.002 },
        "send": { "count": 931, "bytes": 1804412, "limit": 1000, "saturation": 0.931, "oldestAgeSeconds": 86112, "deliveredUnacked": 931 }
      }
    ],
    "snapshotAt": "2026-09-21T10:04:12Z",
    "truncated": false,
    "nextCursor": "c2F0OjAuOTMxOmFsaWNl"
  }
}
```

Alice's send queue is nearly full, and every one of its messages has been delivered but not deleted: a recipient Alice talks to is receiving her messages and failing to acknowledge them. [`messaging/queue/status`](../../status/0.1/spec.md) with `includePeers` names that recipient.

## Security & Privacy

### Data carried

The request carries ranking parameters only. The response carries, per account, its identifier, role, and queue depths — which accounts are active and how much they hold, but no message content and no counterparty identity. A mediator **MUST NOT** place message metadata or content in `ext`.

### Correlation

Account identifiers are the mediator's own (a DID or its stable hash, per [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid)), and the same identifier appears across every `messaging/*` task by design, so an administrator can join a ranking to any other account read. Repeated rankings reveal each account's activity over time. Both are inherent to the task and visible only to an administrator.

### Retention

Transient. A ranking is superseded by the next survey. A console need not keep it beyond the session; the mediator need not retain the request beyond its replay window.

### Consent/purpose

The data is collected to find and relieve queue back-pressure at the mediator. It is not a record of any account's communications and should not be reused as one.
