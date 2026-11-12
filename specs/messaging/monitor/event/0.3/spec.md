---
slug: messaging/monitor/event
version: "0.3"
extends: "0.2"
title: "Messaging — Traffic Monitor Event"
summary: "One-way batch of traffic-metadata events — and, for a subscription that opted in, mux membership events — pushed by a mediator to a live monitor subscriber, sequence-numbered so gaps are detectable and heartbeated so a dead tap is visible."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords: [messaging, monitor, event, traffic, push, observability, mux, membership]
parties:
  - role: Mediator
    requirement: REQUIRED
    member: issuer
  - role: Subscriber
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: RECOMMENDED
  rationale: >-
    Batches are produced by the mediator at high frequency and delivered over a channel that already authenticates the mediator as sender; a display-only consumer gains little from a per-batch signature, but a consumer that persists events or acts on them SHOULD require one.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A batch has no response, so a subscriber cannot detect a replayed or re-delivered batch by correlating a reply; issuedAt, alongside seq, places each batch in time so a stale one is recognisable and a stalled stream is distinguishable from a quiet one.
sideEffects:
  level: none
  rationale: >-
    The subscriber displays or records the events; receiving a batch changes nothing at the mediator and nothing about any message or connection.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The batch carries traffic metadata into the subscriber — counterparties, timing, sizes, protocols, refusal codes — and, where the subscription opted in, mux membership metadata — which member DID attached to or detached from which principal's connection, over which wire, through a grant or an attach, and why. Never a message body.
retention:
  class: transient
  rationale: >-
    Events are live display data for a troubleshooting session; a subscriber that keeps them has chosen to build a traffic log.
errorCodes: []
related:
  - messaging/monitor/subscribe
  - messaging/monitor/unsubscribe
  - messaging/mux/event
  - messaging/mux/connections
---

## Abstract

The **Messaging — Traffic Monitor Event** Trust Task is the push half of the traffic monitor opened by [`messaging/monitor/subscribe`](../../subscribe/0.2/spec.md). The mediator sends batches of [`MonitorEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorEvent)s: arrivals, deliveries, forwards, refusals with their codes, expiries and deletions, each tagged with channel and protocol. A subscription that opted in also receives [`MonitorMembershipEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorMembershipEvent)s: a member DID attached to or detached from a principal's mux connection, with the reason. They travel over the subscriber's live connection. There is no response.

The batch envelope is designed so that the subscriber never has to infer the tap's health:

- `seq` makes lost batches visible.
- `dropped` and `membershipDropped` make discarded events visible.
- A heartbeat makes a dead stream visible.
- `expiresAt` makes the approaching lease end visible.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.2

- **`membership`** (optional) is added: up to 500 [`MonitorMembershipEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorMembershipEvent)s, present on every batch of a subscription that opted in with `messaging/monitor/subscribe` 0.2's `membership`, and on no other.
- **`membershipDropped`** (optional) is added, present exactly when `membership` is: the membership events discarded since the previous batch.
- `events` and `dropped` are unchanged and remain traffic events only. A membership event is not a step in the life of a message, so it is a separate shape in a separate array rather than a new `TrafficStage`: `MonitorEvent` keeps its required `direction`, `stage`, `channel` and `protocol`, none of which a membership change has, and `TrafficStage` stays closed.
- `extends: "0.2"`: a 0.2 consumer reads a 0.3 batch as a 0.2 batch, ignoring the two new members. Since the mediator sends them only to a subscription that asked for them, a consumer that never opts in sees no difference.

The membership definitions are additive `$defs` in `messaging/_shared/0.3/mediator-ops`, the component this version and 0.2 both reference.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the mediator) **MUST**:

1. Emit documents whose `type` is `https://trusttasks.org/spec/messaging/monitor/event/0.3`, with itself as `issuer`, the subscriber as `recipient`, and an `issuedAt`. It **SHOULD** include a `proof`.
2. Number batches per subscription starting at `seq: 1`, increasing by exactly 1 per batch sent, heartbeats included.
3. Include in `events` only traffic events that match the subscription's effective filter, in the order the mediator observed them, at most 500 per batch, and none at all when the subscription turned traffic events off.
4. Report in `dropped` the matching traffic events discarded since the previous batch, whether because of the rate ceiling, a backlog bound, or the subscriber not being live.
5. Carry the subscription's current expiry in `expiresAt`.
6. Obey the delivery rules of `messaging/monitor/subscribe`: send over the subscriber's live connection only, never store a batch in the subscriber's receive queue, never emit events about the delivery of monitor batches, never include a message body, and never delay message delivery to produce a batch.
7. On a subscription whose effective settings include a membership filter, and on no other, carry `membership` and `membershipDropped` on every batch, heartbeats included. Never send them to a subscription that did not opt in.
8. Include in `membership` one event per member change the mediator reports in an `attached` or `detached` [`messaging/mux/event`](../../../mux/event/0.1/spec.md), matching the effective membership filter, in the order the mediator observed them, at most 500 per batch. Each carries the change, the connection's principal and the member, the connection's wire, how the member rides it (`grant` or `attach`), and the reason that `messaging/mux/event` entry carries — an `AttachReason` value for `attached`, a `DetachReason` value for `detached`. A stored grant that did not attach on connect, which `messaging/mux/event` reports `detached`, is reported `detached` here too.
9. Report in `membershipDropped` the matching membership events discarded since the previous batch, for the same reasons as rule 4. The subscription's rate ceiling covers traffic and membership events together.
10. Never carry in a membership event a message, a signed grant or attach statement, a connection's `muxNonce`, a scope, or any key material.

A conforming producer **SHOULD** flush pending events at least once per second, and **SHOULD** send an empty heartbeat batch (`events: []`, and `membership: []` where it applies) at least every 30 seconds while there is nothing to report.

A conforming **consumer** (the subscriber) **MUST**:

1. Ignore any batch whose `subscriptionId` it does not hold or has ended.
2. Treat a `seq` greater than the last seen plus 1 as lost batches, and surface that gap to its user. It **MUST NOT** present the stream as complete across a gap.
3. Surface a non-zero `dropped` or `membershipDropped` rather than hide it.
4. Treat the absence of any batch, events or heartbeat, for longer than the heartbeat interval as a stalled tap, and show it as such. It **MUST NOT** display silence as "no traffic".
5. Not respond. There is no `#response` form for this task.

A consumer **SHOULD** renew the subscription before `expiresAt` if it wants the tap to continue. A consumer that shows membership events **MUST NOT** present them as a connection's member set: they are changes observed while the tap was live, and [`messaging/mux/connections`](../../../mux/connections/0.1/spec.md) is the administrator's view of who rides a connection now.

`events` and `membership` are each in observation order; the two are not interleaved, and a consumer that needs one timeline merges them by `occurredAt`.

## Authorization

The mediator sends a batch only because the recipient holds a live subscription. That subscription, authorized under the rules of `messaging/monitor/subscribe`, is the whole of the entitlement. The effective filters chosen there — for traffic and for membership — already bound what the subscriber may see. The batch asserts nothing the subscriber must act on, so it needs no authority of its own from the subscriber's side. On the consumer side, authenticity comes from the channel, which authenticates the mediator as sender, and optionally from the `proof`. Consumer rule 1 ensures that a batch the subscriber did not ask for is discarded.

## Definitions

- **`subscriptionId`** — the subscription this batch belongs to.
- **`seq`** — batch sequence number: 1, 2, 3, … with no gaps.
- **`events`** — up to 500 [`MonitorEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorEvent)s in observation order. Empty on a heartbeat. Events for one message share its `msgId` once it has one, so a client can follow a message from `received` through `stored` to `delivered` and then `deleted`.
- **`dropped`** — matching traffic events discarded since the previous batch.
- **`membership`** — on an opted-in subscription only: up to 500 [`MonitorMembershipEvent`](../../../_shared/0.3/mediator-ops.schema.json#/$defs/MonitorMembershipEvent)s in observation order, each with `occurredAt`, `change` (`attached` or `detached`), `principal`, `member`, `wire` (`didcomm` for a `didcomm-mux` socket, `tsp` for a `tsp-mux` socket), `source` (`grant` or `attach`), `reason`, and optionally the connection's `sessionId`.
- **`membershipDropped`** — present exactly when `membership` is: matching membership events discarded since the previous batch.
- **`expiresAt`** — the subscription's current lease expiry.

## Request

The mediator pushes a batch to the subscriber over the subscriber's live connection. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A refused TSP frame and a delayed DIDComm delivery

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e01",
  "type": "https://trusttasks.org/spec/messaging/monitor/event/0.3",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T11:02:14Z",
  "payload": {
    "subscriptionId": "mon_8f3a2c1e9b7d4a60",
    "seq": 42,
    "events": [
      {
        "occurredAt": "2026-09-21T11:02:13.418Z",
        "direction": "inbound",
        "stage": "refused",
        "channel": "websocket",
        "protocol": "tsp",
        "from": "did:web:bob.example",
        "to": "did:web:alice.example",
        "size": 912,
        "outcome": {
          "code": "authorization.acl.denied",
          "detail": "alice's access list does not admit bob"
        }
      },
      {
        "occurredAt": "2026-09-21T11:02:13.902Z",
        "direction": "outbound",
        "stage": "delivered",
        "channel": "websocket",
        "protocol": "didcomm",
        "msgId": "1726916533000-0",
        "to": "did:web:alice.example",
        "size": 1843,
        "latencyMs": 4210
      }
    ],
    "dropped": 0,
    "expiresAt": "2026-09-21T11:10:00Z"
  }
}
```

### A member taken over by a newer connection

The subscription opted in to membership events and turned traffic events off.

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e03",
  "type": "https://trusttasks.org/spec/messaging/monitor/event/0.3",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:12:41Z",
  "payload": {
    "subscriptionId": "mon_2b7c9d0e1f3a4b5c",
    "seq": 7,
    "events": [],
    "dropped": 0,
    "membership": [
      {
        "occurredAt": "2026-11-13T09:12:40.611Z",
        "change": "attached",
        "principal": "did:webvh:QmTab1et:tablet.example",
        "member": "did:webvh:QmA1ice:alice.example:work",
        "wire": "tsp",
        "source": "attach",
        "reason": "attach",
        "sessionId": "ses_Q4nB8rT1wZ6y"
      },
      {
        "occurredAt": "2026-11-13T09:12:40.614Z",
        "change": "detached",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "member": "did:webvh:QmA1ice:alice.example:work",
        "wire": "didcomm",
        "source": "grant",
        "reason": "superseded",
        "sessionId": "ses_7Hq2xV9mK3pL"
      }
    ],
    "membershipDropped": 0,
    "expiresAt": "2026-11-13T09:20:00Z"
  }
}
```

The tablet's session attach took the DID's live traffic, so the phone's connection, which carries it through a stored grant, reports it `superseded`; it stays there inactive.

### A heartbeat

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e02",
  "type": "https://trusttasks.org/spec/messaging/monitor/event/0.3",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T11:02:44Z",
  "payload": {
    "subscriptionId": "mon_8f3a2c1e9b7d4a60",
    "seq": 43,
    "events": [],
    "dropped": 0,
    "expiresAt": "2026-09-21T11:10:00Z"
  }
}
```

## Security & Privacy

### Data carried

A batch carries traffic metadata only: counterparties where the mediator knows them, timestamps, sizes, channels, protocols, refusal codes and short details, and message types only for traffic addressed to the mediator itself. On an opted-in subscription it also carries membership metadata: the principal and member DIDs of a mux connection, its wire and session identifier, how the member rides it and why it joined or left. A producer **MUST NOT** include any message body, any part of a ciphertext, any signed grant or attach statement, any connection nonce, or any key material. `outcome.detail` is free text, and a producer **MUST NOT** put message content or secrets in it.

### Correlation

Events are meant to be joined. `msgId` links the steps of one message, and the counterparties link accounts in time. For an administrator's unfiltered tap, a stream of batches is a live communication graph of the mediator. Membership events add the association mux exists to carry — which DIDs one principal holds, and when each joined or left — which is why they are sent only to a subscription that asked for them, under the narrowing rules of `messaging/monitor/subscribe`. An observer of the subscriber's connection learns batch timing and size, which roughly tracks traffic volume. Heartbeats at a fixed interval reveal nothing beyond the subscription's existence.

### Retention

Batches are transient display data. A subscriber **SHOULD NOT** persist them beyond the troubleshooting session. One that does has built a traffic log — and, with membership events, a log of whose DIDs travel together — and takes on the retention and access obligations that come with it. The mediator retains nothing per batch.

### Consent/purpose

The purpose is live diagnosis of message flow, and of mux membership, for whoever holds the subscription. Using the stream for surveillance, analytics, or any purpose beyond troubleshooting is outside this task. Whether such uses are permissible is the mediator operator's policy, not this specification's.
