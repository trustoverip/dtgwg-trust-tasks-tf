---
slug: messaging/monitor/subscribe
version: "0.2"
extends: "0.1"
title: "Messaging — Subscribe to Traffic Monitor"
summary: "Open or renew a leased, filtered, live tap on a mediator's traffic metadata — what arrives, what leaves, over DIDComm or TSP, and what was refused and why — and, only when asked, on mux membership changes, delivered as messaging/monitor/event batches."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords: [messaging, monitor, subscribe, traffic, troubleshooting, observability, mux, membership]
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
    The stream that follows discloses traffic metadata: which accounts exchange messages, when, how large, over which protocol, and why frames were refused; and, only on a subscription that opts in, which member DIDs attach to and detach from which principals' mux connections, and why. For an administrator this covers every account at the mediator. No message body is ever disclosed.
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
  - messaging/mux/event
  - messaging/mux/connections
---

## Abstract

The **Messaging — Subscribe to Traffic Monitor** Trust Task opens a live, filtered tap on what is moving through a mediator: frames arriving and leaving, messages stored, delivered, forwarded, expired or deleted, and frames refused along with the reason. Each step is tagged with the channel it used (websocket, REST, peer mediator) and the wire protocol it carried (DIDComm v2, DIDComm v1, TSP). Its purpose is troubleshooting. It answers "is my message reaching the mediator, and what happens to it there?" without anyone reading server logs.

A subscription may also **opt in** to mux membership events: a member DID attached to or detached from a principal's `didcomm-mux` or `tsp-mux` connection, through a grant or a session attach, with the reason. They are never sent unless the request asks for them with `membership`, and a subscription that wants only them turns traffic events off with `traffic: false`.

Events arrive as [`messaging/monitor/event`](../../event/0.3/spec.md) batches over the subscriber's live connection. A subscription is a **lease**. It lapses unless renewed, so an abandoned console cannot leave a tap running. It ends early on [`messaging/monitor/unsubscribe`](../../unsubscribe/0.1/spec.md).

The stream carries **metadata only** ([`MonitorEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorEvent), [`MonitorMembershipEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorMembershipEvent)). It never carries a message body.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

- **`membership`** (optional) is added to the request and the response: a [`MembershipFilter`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MembershipFilter) whose presence opts the subscription in to mux membership events. Absent, as in every 0.1 request, there are none.
- **`traffic`** (optional) is added to the request and the response: `false` turns traffic events off, for a subscription that wants membership events alone. Absent, traffic events are on, as in 0.1.
- `filter` now references `MonitorFilter` in `messaging/_shared/0.3/mediator-ops`, which is the same shape as in `_shared/0.1`; the move only puts the traffic and membership filters in one component.
- `extends: "0.1"`: a 0.1 consumer reads a 0.2 subscribe as a 0.1 subscribe, ignoring `membership` and `traffic`, and answers without them — which Consumer rule 5 makes the subscriber's signal that it gets no membership events and does get traffic.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the subscriber) **MUST**:

1. Emit a document whose `type` is `https://trusttasks.org/spec/messaging/monitor/subscribe/0.2`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, and a `proof`.
2. Omit `subscriptionId` to open a new subscription. To renew or re-filter one it holds, include that subscription's `subscriptionId`. A renewal replaces the filters, lease and rate with the values in the new request; a renewal without `membership` ends the subscription's membership events.
3. Renew before the `expiresAt` it was last given, if it wants the tap to continue.
4. Not send `filter` with `traffic: false`, nor `traffic: false` without `membership`.
5. Take the response, not its own request, as what it will see: treat an absent `membership` in the response as "no membership events", and an absent `traffic` as "traffic events on".

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`. Respond `malformedRequest` to `filter` with `traffic: false`, or to `traffic: false` without `membership`.
2. Where `subscriptionId` is present and names no live subscription held by this requester, respond `messaging/monitor/subscribe:unknownSubscription`. A subscription is renewable only by the requester that opened it.
3. Where a new subscription would exceed its per-requester limit, respond `messaging/monitor/subscribe:tooManySubscriptions`.
4. Apply the [Authorization](#authorization) rules to both filters, then return the **effective** traffic filter, the effective membership filter where the subscription receives membership events, `traffic: false` where it receives no traffic events, the lease's `expiresAt` (default 300 seconds, at most 3600 seconds from now), and the rate ceiling in force, which **MAY** be lower than the one requested. With traffic events off, the effective traffic `filter` is the empty filter.
5. Send membership events only to a subscription whose request carried `membership`, and omit `membership` from the response of every other — including one that asked while mux is not enabled at the mediator, where there will be none to send. It **MUST NOT** return a membership filter for a subscription it will send no membership events.

While a subscription is live, the mediator **MUST** deliver matching events according to these rules:

6. **Live connection only.** Events are sent as `messaging/monitor/event` documents over the subscriber's live connection (for example its websocket, or a live TSP channel). They **MUST NOT** be stored in the subscriber's receive queue. A monitor must never fill the queue it is observing.
7. **Drop, don't buffer.** While the subscriber has no live connection, matching events are counted and reported in the next batch's `dropped` or `membershipDropped`. The mediator **MAY** hold a small bounded backlog to cover brief reconnects, and **MUST NOT** accumulate events without bound.
8. **No feedback loop.** The mediator **MUST NOT** emit events describing the delivery of `messaging/monitor/event` documents themselves, to this subscriber or any other.
9. **Never slow the data plane.** Monitoring **MUST NOT** delay, back-pressure, or fail the delivery of any message, or any attach, detach or revocation of a mux member. When the rate ceiling is exceeded, or event production falls behind, events are discarded and counted.
10. **Metadata only.** An event **MUST NOT** carry a message body or any part of one. `messageType` appears only when the mediator itself was the addressee and read the message (a routing forward, a pickup request, a Trust Task type URI, a trust ping). End-to-end-encrypted traffic the mediator merely relays has no `messageType`. A membership event carries no signed statement, nonce, scope or key material.
11. **End cleanly.** A subscription ends at `expiresAt` or on `messaging/monitor/unsubscribe`. After that the mediator sends no further events for it.

## Authorization

The authority is **standing over the traffic and the connections observed**.

- **Administrators** (`admin`, `rootAdmin`) may subscribe with any traffic filter, including none, and any membership filter, including the empty one. Observing the whole mediator's traffic metadata and mux membership is an operator capability.
- **Any other account** may observe only its own traffic: events whose `from` or `to` is itself. When its filter omits `dids`, the mediator narrows `dids` to `[requester]` and returns that as the effective filter. When its filter names any other account in `dids`, the mediator responds `permissionDenied`. It does not silently narrow, because a filter the subscriber did not ask for would show an empty tap that looks like silence.
- **Any other account** may likewise observe only membership changes it is party to: events whose `principal` or `member` is itself — the changes [`messaging/mux/event`](../../../mux/event/0.1/spec.md) already reports to it as principal, and those of its own DID as a member. The same narrowing applies to the membership filter's `dids`: omitted, it is narrowed to `[requester]`; naming another account, the request is refused with `permissionDenied`.

The effective filters in the response are the mediator's statement of what the subscriber will see, and the subscriber **SHOULD** display them. The `proof` establishes who opened the tap and makes that auditable. Entitlement comes from the mediator's own account records.

## Definitions

- **`filter`** — OPTIONAL. A [`MonitorFilter`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorFilter) for traffic events. Its members are combined with AND. Values within one member are combined with OR. Absent or empty selects every traffic event the requester is entitled to see.
- **`traffic`** — OPTIONAL. `false` turns traffic events off; absent or `true` leaves them on.
- **`membership`** — OPTIONAL. A [`MembershipFilter`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MembershipFilter): its presence opts in to membership events, and its members — `dids` (principal or member), `changes` (`attached`, `detached`), `wires` (`didcomm`, `tsp`), `sources` (`grant`, `attach`) — narrow them, ANDed, with values ORed. `{}` selects every membership event the requester is entitled to see. Absent selects none.
- **`leaseSeconds`** — OPTIONAL. The requested lease, from 10 to 3600 seconds. The default is 300.
- **`maxEventsPerSecond`** — OPTIONAL. The most the subscriber can absorb, traffic and membership events together. The mediator applies this value or a lower one, and reports the one in force.
- **`subscriptionId`** — OPTIONAL in the request, and present in the response. The handle for renewal, unsubscribe, and matching `messaging/monitor/event` batches. The mediator chooses it, and it **SHOULD** be unguessable.
- **`expiresAt`** (response) — when the lease lapses without renewal.

Membership events are opt-in, and traffic events are not, for one reason: a 0.1 subscription receives traffic events with no member asking for them, and 0.2 keeps that so that it can extend 0.1. Everything 0.2 adds is off until asked for.

## Request

The subscriber sends the request to the mediator whose traffic it wants to watch. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator watches refused TSP and DIDComm traffic for one account

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c01",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.2",
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
    "proofPurpose": "authentication",
    "proofValue": "z5mc..."
  }
}
```

### An administrator watches one DID's mux membership, and nothing else

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c03",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.2",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-13T09:00:00Z",
  "payload": {
    "traffic": false,
    "membership": {
      "dids": ["did:webvh:QmA1ice:alice.example:work"]
    },
    "leaseSeconds": 1200
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-13T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4kd..."
  }
}
```

## Response

The mediator responds with the subscription handle, lease and effective settings, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### The subscription as granted

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c02",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.2#response",
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

The mediator granted the filter unchanged but lowered the rate ceiling from 50 to 20 events per second. The response carries no `membership`, so the subscription receives no membership events.

### A membership-only subscription as granted

```json
{
  "id": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c04",
  "type": "https://trusttasks.org/spec/messaging/monitor/subscribe/0.2#response",
  "threadId": "urn:uuid:9d4e2c71-0a6b-4f3e-8c21-5e7f1a2b3c03",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:00:00Z",
  "payload": {
    "subscriptionId": "mon_2b7c9d0e1f3a4b5c",
    "expiresAt": "2026-11-13T09:20:00Z",
    "filter": {},
    "traffic": false,
    "membership": {
      "dids": ["did:webvh:QmA1ice:alice.example:work"]
    },
    "maxEventsPerSecond": 50
  }
}
```

## Security & Privacy

### Data carried

The request carries a traffic filter and a membership filter, either of which may name accounts. The event stream the subscription opens carries traffic metadata: counterparties, timing, sizes, protocols, channels, refusal codes, and, for mediator-addressed traffic, message types; and, when asked for, membership metadata: principal and member DIDs, wire, session identifier, source and reason. For an administrator this is the communication graph of every account at the mediator, and the map of which DIDs travel together. That is the reason for the proof requirement, the lease, and membership events being opt-in. No message body is ever carried, and a producer **MUST NOT** place anything but filter criteria in `ext`.

### Correlation

The stream exists to correlate. It joins a `msgId` across its stored, delivered and deleted steps, and ties accounts to each other in time. Membership events tie member DIDs to the principals carrying them — the association a mux connection makes — which is why no subscription receives them without asking. An observer of the subscriber's connection sees event volume and timing even when the content is encrypted. A non-admin subscriber sees correlations only among its own traffic and its own delegations.

### Retention

The mediator keeps the subscription only for its lease and **SHOULD** audit its opening, including both effective filters. The subscriber **SHOULD** treat received events as transient display data. A console that persists them has made a traffic log, and inherits the retention obligations of one.

### Consent/purpose

The purpose is live troubleshooting of message flow and of mux membership: an operator diagnosing the mediator, or an account diagnosing its own delivery and its own delegations. Retaining or mining the stream for other purposes is outside this task's purpose. Whether opening an unfiltered administrative tap needs further approval is the mediator operator's policy, not this specification's.
