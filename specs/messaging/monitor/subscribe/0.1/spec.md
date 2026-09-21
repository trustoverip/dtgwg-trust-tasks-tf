---
slug: messaging/monitor/subscribe
version: "0.1"
title: "Messaging — Subscribe to Traffic Monitor"
summary: "Open or renew a leased, filtered, live tap on a mediator's traffic metadata — what arrives, what leaves, over DIDComm or TSP, and what was refused and why — delivered as messaging/monitor/event batches."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords: [messaging, monitor, subscribe, traffic, troubleshooting, observability]
parties:
  - role: Subscriber (an administrator, or an account watching its own traffic)
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    A subscription causes the mediator to stream traffic metadata — for an administrator, about every account — to the subscriber for the life of a lease; who opened that stream, and with what filter, must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Opening a subscription creates server-side state, which makes the task consequential (SPEC §7.3 item 17); a replayed subscribe must fall outside a bounded window rather than silently re-open a tap its owner closed.
sideEffects:
  level: mutating
  rationale: >-
    Creates, renews, or re-filters a leased subscription at the mediator. Reversible — it ends at its expiry or on messaging/monitor/unsubscribe — and changes nothing about any message.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The stream that follows discloses traffic metadata: which accounts exchange messages, when, how large, over which protocol, and why frames were refused. For an administrator this covers every account at the mediator. No message body is ever disclosed.
retention:
  class: exchange
  rationale: >-
    The mediator holds the subscription only for its lease; nothing survives its expiry or unsubscribe.
errorCodes:
  - code: messaging/monitor/subscribe:unknownSubscription
    meaning: The request names a `subscriptionId` the mediator does not hold for this requester — it never existed, has expired, was ended, or belongs to someone else. Omit `subscriptionId` to open a new subscription.
    retryable: false
  - code: messaging/monitor/subscribe:tooManySubscriptions
    meaning: The requester already holds the maximum number of concurrent subscriptions the mediator allows. End one, or wait for one to expire.
    retryable: true
related:
  - messaging/monitor/event
  - messaging/monitor/unsubscribe
  - messaging/stats/show
---

## Abstract

The **Messaging — Subscribe to Traffic Monitor** Trust Task opens a live, filtered tap on what is moving through a mediator: frames arriving and leaving, messages stored, delivered, forwarded, expired or deleted, and frames refused along with the reason. Each step is tagged with the channel it used (websocket, REST, peer mediator) and the wire protocol it carried (DIDComm v2, DIDComm v1, TSP). Its purpose is troubleshooting. It answers "is my message reaching the mediator, and what happens to it there?" without anyone reading server logs.

Events arrive as [`messaging/monitor/event`](../../event/0.1/spec.md) batches over the subscriber's live connection. A subscription is a **lease**. It lapses unless renewed, so an abandoned console cannot leave a tap running. It ends early on [`messaging/monitor/unsubscribe`](../../unsubscribe/0.1/spec.md).

The stream carries **metadata only** ([`MonitorEvent`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MonitorEvent)). It never carries a message body.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the subscriber) **MUST**:

1. Emit a document whose `type` is `https://trusttasks.org/spec/messaging/monitor/subscribe/0.1`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, and a `proof`.
2. Omit `subscriptionId` to open a new subscription. To renew or re-filter one it holds, include that subscription's `subscriptionId`. A renewal replaces the filter, lease and rate with the values in the new request.
3. Renew before the `expiresAt` it was last given, if it wants the tap to continue.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where `subscriptionId` is present and names no live subscription held by this requester, respond `messaging/monitor/subscribe:unknownSubscription`. A subscription is renewable only by the requester that opened it.
3. Where a new subscription would exceed its per-requester limit, respond `messaging/monitor/subscribe:tooManySubscriptions`.
4. Apply the [Authorization](#authorization) rules to the filter, then return the **effective** filter, the lease's `expiresAt` (default 300 seconds, at most 3600 seconds from now), and the rate ceiling in force, which **MAY** be lower than the one requested.

While a subscription is live, the mediator **MUST** deliver matching events according to these rules:

5. **Live connection only.** Events are sent as `messaging/monitor/event` documents over the subscriber's live connection (for example its websocket, or a live TSP channel). They **MUST NOT** be stored in the subscriber's receive queue. A monitor must never fill the queue it is observing.
6. **Drop, don't buffer.** While the subscriber has no live connection, matching events are counted and reported in the next batch's `dropped`. The mediator **MAY** hold a small bounded backlog to cover brief reconnects, and **MUST NOT** accumulate events without bound.
7. **No feedback loop.** The mediator **MUST NOT** emit events describing the delivery of `messaging/monitor/event` documents themselves, to this subscriber or any other.
8. **Never slow the data plane.** Monitoring **MUST NOT** delay, back-pressure, or fail the delivery of any message. When the rate ceiling is exceeded, or event production falls behind, events are discarded and counted in `dropped`.
9. **Metadata only.** An event **MUST NOT** carry a message body or any part of one. `messageType` appears only when the mediator itself was the addressee and read the message (a routing forward, a pickup request, a Trust Task type URI, a trust ping). End-to-end-encrypted traffic the mediator merely relays has no `messageType`.
10. **End cleanly.** A subscription ends at `expiresAt` or on `messaging/monitor/unsubscribe`. After that the mediator sends no further events for it.

## Authorization

The authority is **standing over the traffic observed**.

- **Administrators** (`admin`, `rootAdmin`) may subscribe with any filter, including none. Observing the whole mediator's traffic metadata is an operator capability.
- **Any other account** may observe only its own traffic: events whose `from` or `to` is itself. When its filter omits `dids`, the mediator narrows `dids` to `[requester]` and returns that as the effective filter. When its filter names any other account in `dids`, the mediator responds `permissionDenied`. It does not silently narrow, because a filter the subscriber did not ask for would show an empty tap that looks like silence.

The effective filter in the response is the mediator's statement of what the subscriber will see, and the subscriber **SHOULD** display it. The `proof` establishes who opened the tap and makes that auditable. Entitlement comes from the mediator's own account records.

## Definitions

- **`filter`** — OPTIONAL. A [`MonitorFilter`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MonitorFilter). Its members are combined with AND. Values within one member are combined with OR. Absent or empty selects everything the requester is entitled to see.
- **`leaseSeconds`** — OPTIONAL. The requested lease, from 10 to 3600 seconds. The default is 300.
- **`maxEventsPerSecond`** — OPTIONAL. The most the subscriber can absorb. The mediator applies this value or a lower one, and reports the one in force.
- **`subscriptionId`** — OPTIONAL in the request, and present in the response. The handle for renewal, unsubscribe, and matching `messaging/monitor/event` batches. The mediator chooses it, and it **SHOULD** be unguessable.
- **`expiresAt`** (response) — when the lease lapses without renewal.

## Request

The subscriber sends the request to the mediator whose traffic it wants to watch. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator watches refused TSP and DIDComm traffic for one account

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c01",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T11:00:00Z",
  "payload": {
    "filter": {
      "dids": ["did:web:alice.example"],
      "protocols": ["didcomm", "tsp"],
      "failuresOnly": true
    },
    "leaseSeconds": 600,
    "maxEventsPerSecond": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T11:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5mc..."
  }
}
```

## Response

The mediator responds with the subscription handle, lease and effective settings, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### The subscription as granted

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c02",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.1#response",
  "threadId": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T11:00:00Z",
  "payload": {
    "subscriptionId": "mon_8f3a2c1e9b7d4a60",
    "expiresAt": "2026-09-21T11:10:00Z",
    "filter": {
      "dids": ["did:web:alice.example"],
      "protocols": ["didcomm", "tsp"],
      "failuresOnly": true
    },
    "maxEventsPerSecond": 20
  }
}
```

The mediator granted the filter unchanged but lowered the rate ceiling from 50 to 20 events per second.

## Security & Privacy

### Data carried

The request carries a filter, which may name accounts. The event stream the subscription opens carries traffic metadata: counterparties, timing, sizes, protocols, channels, refusal codes, and, for mediator-addressed traffic, message types. For an administrator this is the communication graph of every account at the mediator. That is the reason for the proof requirement and the lease. No message body is ever carried, and a producer **MUST NOT** place anything but filter criteria in `ext`.

### Correlation

The stream exists to correlate. It joins a `msgId` across its stored, delivered and deleted steps, and ties accounts to each other in time. An observer of the subscriber's connection sees event volume and timing even when the content is encrypted. A non-admin subscriber sees correlations only among its own traffic.

### Retention

The mediator keeps the subscription only for its lease and **SHOULD** audit its opening, including the effective filter. The subscriber **SHOULD** treat received events as transient display data. A console that persists them has made a traffic log, and inherits the retention obligations of one.

### Consent/purpose

The purpose is live troubleshooting of message flow: an operator diagnosing the mediator, or an account diagnosing its own delivery. Retaining or mining the stream for other purposes is outside this task's purpose. Whether opening an unfiltered administrative tap needs further approval is the mediator operator's policy, not this specification's.
