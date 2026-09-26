---
slug: messaging/monitor/unsubscribe
version: "0.1"
title: "Messaging — Unsubscribe from Traffic Monitor"
summary: "End a traffic-monitor subscription before its lease lapses, and learn how many events it delivered and dropped."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Subscriber (the subscription's creator, or a rootAdmin)
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Ending a subscription changes server-side state on the subscriber's behalf — and a rootAdmin may end one it does not own — so the request must be attributable to whoever issued it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task mutates state, which makes it consequential (SPEC §7.3 item 17); a replayed unsubscribe must fall outside a bounded window so it cannot end a later subscription that happens to reuse nothing but its requester.
sideEffects:
  level: mutating
  rationale: >-
    Ends a leased monitor subscription. Low impact and self-healing — the subscriber can open a new one at once — and no message is touched.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: none
retention:
  class: transient
  rationale: >-
    Once the subscription has ended nothing of it is retained beyond the mediator's audit record.
errorCodes:
  - code: messaging/monitor/unsubscribe:unknownSubscription
    meaning: No live subscription with this `subscriptionId` exists that the requester may end — it never existed, has already expired or ended, or belongs to another requester and the requester is not a rootAdmin.
    retryable: false
related:
  - messaging/monitor/subscribe
  - messaging/monitor/event
---

## Abstract

The **Messaging — Unsubscribe from Traffic Monitor** Trust Task ends a [`messaging/monitor/subscribe`](../../subscribe/0.1/spec.md) subscription before its lease lapses. A console calls it on exit, so the mediator stops producing events at once rather than when the lease runs out. The response reports the subscription's lifetime totals of events sent and dropped, which tells the operator whether the tap they just closed was complete.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the subscriber) **MUST** emit a document whose `type` is `https://trusttasks.org/spec/messaging/monitor/unsubscribe/0.1`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, a `proof`, and the `subscriptionId` to end.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where no live subscription with `subscriptionId` exists that the requester may end, respond `messaging/monitor/unsubscribe:unknownSubscription`. The response is the same for a subscription that never existed and one owned by someone else, so a non-owner learns nothing about other subscriptions.
3. End the subscription, send no further `messaging/monitor/event` batches for it, and respond with its lifetime `eventsSent` and `eventsDropped`.

## Authorization

The authority is **ownership of the subscription**: the requester that opened it may end it. A `rootAdmin` may also end any subscription. This is the operator's way to close a tap left running by another administrator or account, for example one whose monitoring load is hurting the mediator. An `admin` may end only its own. The `proof` identifies who ended the subscription, and the mediator **SHOULD** audit a subscription ended by someone other than its creator.

## Definitions

- **`subscriptionId`** — REQUIRED. The handle returned by `messaging/monitor/subscribe`.
- **`eventsSent`** (response) — events delivered to the subscriber over the subscription's life.
- **`eventsDropped`** (response) — matching events discarded over its life, because of the rate ceiling or because the subscriber was not live. A non-zero value means the tap was incomplete.

## Request

The subscriber sends the request to the mediator that holds the subscription. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A console closing its tap on exit

```json
{
  "id": "urn:uuid:4a7b9c2d-6e1f-4a3b-9c8d-1e2f3a4b5c01",
  "type": "https://trusttasks.org/spec/messaging/monitor/unsubscribe/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T11:07:30Z",
  "payload": {
    "subscriptionId": "mon_8f3a2c1e9b7d4a60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T11:07:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z2pn..."
  }
}
```

## Response

The mediator confirms the subscription has ended and reports its totals, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### Totals for the closed tap

```json
{
  "id": "urn:uuid:4a7b9c2d-6e1f-4a3b-9c8d-1e2f3a4b5c02",
  "type": "https://trusttasks.org/spec/messaging/monitor/unsubscribe/0.1#response",
  "threadId": "urn:uuid:4a7b9c2d-6e1f-4a3b-9c8d-1e2f3a4b5c01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T11:07:30Z",
  "payload": {
    "eventsSent": 1284,
    "eventsDropped": 17
  }
}
```

## Security & Privacy

### Data carried

The request carries only a subscription handle, and the response only two counters. No traffic metadata and no account identifier moves in either direction.

### Correlation

`subscriptionId` links this request to the subscribe exchange and the event batches it produced. That is its purpose, and it links nothing beyond the requester's own monitoring session. The `unknownSubscription` answer is the same for a missing subscription and a foreign one, so the task cannot be used to probe for other subscribers.

### Retention

Nothing about the ended subscription need be retained beyond the mediator's audit record of its opening and, where applicable, of its ending by a rootAdmin.

### Consent/purpose

The purpose is to stop a monitoring stream promptly and account for its completeness. It carries no data that could be reused for anything else. Whether ending another party's subscription needs further approval is the mediator operator's policy, not this specification's.
