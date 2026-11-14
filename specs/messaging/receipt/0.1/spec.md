---
slug: messaging/receipt
version: "0.1"
title: "Messaging — Receipt"
summary: "When a sender asks for one, the recipient's delivery layer tells it that the message is durably held at the far end, by echoing the delivery id the sender stamped — end-to-end evidence of delivery that works across mediators, where a mediator's status answer cannot."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords:
  - receipt
  - delivery
  - acknowledgement
  - delivery confirmation
  - outbox
parties:
  - role: Recipient (of the receipted message; sends the receipt)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Sender (of the receipted message; receives the receipt)
    requirement: REQUIRED
    member: recipient
    identifierScope: any
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    A receipt settles a delivery only when it comes from the party the receipted message was sent to, authenticated by the same end-to-end binding that carried that message (Consumer rule 1). The transport's authenticated sender is therefore the whole of what is relied on, and a proof would add a signature the sender has no use for beyond the exchange.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A receipt changes what the sender records about a delivery, so SPEC §7.3 item 17 makes issuedAt REQUIRED: the sender places it in its acceptance window and keeps its duplicate record inside that window. A receipt is emitted when a message is durably held, and re-emitted on a duplicate, so a fresh issuedAt is always available.
sideEffects:
  level: mutating
  rationale: >-
    The sender records the confirmed delivery against the message's entry — typically settling it from sent to delivered and ending its retries. The change is idempotent: a second receipt for a settled entry changes nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The receipt carries one identifier the sender itself chose. What it reveals is when the recipient's delivery layer held the message, which is the evidence the sender asked for.
retention:
  class: transient
  rationale: >-
    The sender keeps the fact of delivery on its own outbox entry, not the receipt. The receipt document itself is consumed on arrival; only the duplicate record of SPEC §7.2 item 11 outlives it, for the acceptance window.
errorCodes: []
related:
  - messaging/message/status
  - messaging/wake/request
---

## Abstract

The **Messaging — Receipt** Trust Task is the recipient's half of end-to-end delivery confirmation. A sender that wants to know a message arrived stamps it with a **delivery id** — the TSP binding's `deliveryId` envelope member ([TSP binding 0.2 §2.1](../../../../bindings/tsp/0.2/spec.md#21-delivery-receipts)) or the DIDComm binding's `delivery_id` header ([DIDComm binding §2.1](../../../../bindings/didcomm/0.2/spec.md#21-delivery-receipts)). The recipient's delivery layer, once it durably holds the message, sends back a receipt whose `confirms` echoes that id. The sender settles the matching delivery on it.

A mediator can only answer for messages it holds itself. [`messaging/message/status`](../../message/status/0.2/spec.md) confirms delivery well when sender and recipient share a mediator, but a message the sender's mediator relays to the recipient's own leaves it at once, and the sender can never learn from either mediator that it was collected. A receipt does not depend on any mediator: it comes from the recipient, over the same end-to-end binding, authenticated as the recipient.

A receipt is **requested, never automatic**. A recipient that receipted every message would tell any sender that could reach it when it came online. It receipts only what the sender asked to have receipted, and **MAY** decline.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Requesting a receipt

A party that wants a message receipted stamps a delivery id on it: `deliveryId` in the TSP binding envelope, or the `delivery_id` header in the DIDComm binding. It **MUST** choose a value that identifies the delivery it is tracking, is unique among its deliveries to that recipient, and is not derived from the message's content; it **SHOULD** be unpredictable to anyone else, and **MUST NOT** be reused for a different recipient, so that receipts cannot be joined across recipients. Re-sending the same message on a retry keeps the same delivery id, so a receipt for either attempt settles it.

### Producer (the recipient of the receipted message)

A conforming **producer** of a receipt:

1. **SHOULD** send a receipt for a message that requested one. It **MAY** decline — for a sender it has no relationship with, or wherever telling the sender when it is online is unwanted — and declining is silent.
2. **MUST** send it only for a message that requested one, and only to that message's **transport-authenticated sender**, never to an in-band `issuer` the transport did not authenticate. A message with no authenticated sender gets no receipt.
3. **MUST** send it only once the message is **durably held**: written where a restart will not lose it, and checked against the duplicate-execution record of [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11 where the message is a consequential Trust Task. A message that is a duplicate of one already held is receipted again, since the first receipt may have been lost.
4. **MUST** set `confirms` to the delivery id exactly as received, with itself as `issuer` and the receipted message's sender as `recipient`, and carry it over the binding the message arrived on, or over the highest-preference binding both parties advertise.
5. **MUST NOT** request a receipt for a receipt: a receipt carries no delivery id.

A receipt states that the producer's delivery layer **holds** the message. It says nothing about whether the carried document is valid, accepted, executed or answered — those are the [document lifecycle](/SPEC.md#412-document-lifecycle)'s states, carried by the replies the task itself defines — and a producer **MUST NOT** delay, withhold or vary a receipt according to them.

### Consumer (the sender of the receipted message)

A conforming **consumer** of a receipt:

1. **MUST** settle a delivery on a receipt only where `confirms` equals a delivery id it stamped **and** the receipt's transport-authenticated sender is the party that delivery was addressed to. A receipt from anyone else, or with no authenticated sender, settles nothing.
2. **MUST** ignore, silently, a receipt whose `confirms` names no delivery it is tracking, or one already settled.
3. **MUST NOT** answer a receipt with any document — no `#response`, no courtesy `{}` acknowledgement, and no `trust-task-error`, even when it refuses the receipt. Nobody acknowledges an acknowledgement; a lost receipt costs one re-send, which the recipient absorbs as a duplicate and receipts again.
4. **MUST NOT** treat the absence of a receipt as a failure of delivery on its own. A recipient may not support receipts, or may decline; only a recipient known to send them makes silence evidence of anything.

### Transition from the delivery-layer receipt

Before this specification, the Affinidi messaging delivery layer carried the same evidence as a bare message body, `{"type": "https://affinidi.com/delivery/1.0/receipt", "confirms": "<id>"}`, correlated on the DIDComm `thid`. During the transition:

- A consumer **SHOULD** also accept that form, under exactly the rules above: same correlation, same authenticated-sender check, same silence.
- A producer **SHOULD** answer a message carrying this specification's delivery id with this task. It **MAY** keep sending the earlier form to a sender whose delivery layer asked for one in that layer's own way, and **MUST NOT** send either form for a message that asked for neither.

A later version of this specification will withdraw the earlier form once deployed delivery layers emit this one.

## Authorization

The authority is **being the party the receipted message was addressed to**. A receipt is evidence the sender asked that party for, and it is accepted from that party alone (Consumer rule 1), identified by the end-to-end binding that authenticated it. Nothing else entitles a receipt: the in-band `issuer`, a `proof`, or knowing a delivery id is not enough, because the evidence is about *that* recipient holding *that* message. A receipt authorizes nothing beyond recording it; in particular it never confirms that the carried task was performed.

## Definitions

- **Delivery id** — an opaque string the sender stamps on a message to request a receipt: `deliveryId` in the TSP binding envelope, `delivery_id` in the DIDComm binding headers. At most 256 characters.
- **`confirms`** — REQUIRED. The delivery id being confirmed, exactly as the sender stamped it.
- **Durably held** — written by the recipient's delivery layer where a restart will not lose it, after duplicate detection.

## Request

The recipient of the receipted message sends the receipt to its sender. The payload validates against [`payload.schema.json`](payload.schema.json). The task is fire-and-forget: it defines no response.

### A wallet confirms the credentials a community pushed to it

```json
{
  "id": "urn:uuid:2b8e6d40-91c3-4f5a-8e27-c4d1a0b9f312",
  "type": "https://trusttasks.org/spec/messaging/receipt/0.1",
  "issuer": "did:peer:2.Ez6LSwallet.Vz6Mkwallet",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-11-14T09:30:02Z",
  "payload": {
    "confirms": "dlv_Hk3q9ZpX2mYc7RtW"
  }
}
```

## Security & Privacy

### Data carried

One identifier the sender itself chose. The receipt carries no state, no time beyond its own `issuedAt`, and nothing of the receipted message. A producer **MUST NOT** add a reading state, a content digest or an outcome in `ext`: a receipt is evidence of holding, and anything more belongs to the replies the carried task defines.

### Correlation

A receipt tells the sender when the recipient's delivery layer held its message, which for a device that collects on opening is when the device came online. That is the evidence asked for, and it is why receipts are requested per message rather than sent for everything, and why a recipient **MAY** decline. A recipient that wants to blur the time **MAY** delay its receipt, and **MAY** coarsen `issuedAt` within the sender's acceptance window ([SPEC §13.1](/SPEC.md#131-identifier-correlation-and-linkability)).

Because the sender chooses the delivery id, a sender that reused one across recipients could join their receipts; the rule that a delivery id is never reused for another recipient closes that. An intermediary sees an end-to-end message the size of a receipt and nothing more.

### Retention

The sender records the confirmed delivery on its own entry for the message and keeps that for as long as it keeps the entry. The receipt document itself is not retained beyond the duplicate record of [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11, which holds a digest for the acceptance window. A producer keeps nothing about a receipt it sent.

### Consent/purpose

The purpose is delivery assurance: so a sender can stop re-sending, and stop presenting a delivered message as a failure, on evidence from the recipient rather than from a mediator that cannot see the far end. Using receipt timing to profile when a recipient is active is outside that purpose. That is the reason receipts are requested per message and may be declined, and the reason a recipient's choice to decline is never itself treated as a failure.
