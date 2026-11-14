---
slug: messaging/wake/request
version: "0.1"
title: "Messaging — Wake Request"
summary: "A sender that has just queued mail for a DID asks that DID's own mediator to ring the device — so a party with no standing at the device can get it woken, while the mediator alone decides, nothing about the message travels, and the answer reveals nothing."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords:
  - wake
  - push
  - doorbell
  - approver
  - notification
parties:
  - role: Sender (the party that queued the mail)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator (the addressee's own)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request reaches the addressee's mediator from a party that is usually not one of its accounts, often relayed by another mediator or over an anonymous session, so the session does not reliably name the sender. The mediator binds the wake to mail that sender queued (Consumer rule 3) and rate-limits per sender, and both need the sender established independent of how the bytes arrived.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A captured request replayed later could ring a device again for mail it has already seen. The ids a request names are only fresh for a short window (Consumer rule 2), and the mediator places the request in that window by its issuedAt.
sideEffects:
  level: none
  rationale: >-
    A contentless doorbell. The mediator may ask the gateway to deliver a wake, which changes no state of the addressee's, delivers no message and alters no queue. The only thing it records is that an id has rung, which exists to make the task idempotent: a repeat rings nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response is the same empty object for every request. The request carries a DID and message identifiers the mediator already holds — routing metadata, never message content.
retention:
  class: transient
  rationale: >-
    The mediator keeps a request only while it is armed waiting for its messages to arrive (at most 120 seconds), and keeps a per-message marker that it has rung for as long as the message stays queued, so that a pending message is not rung twice.
errorCodes: []
related:
  - messaging/wake/set
  - messaging/message/status
  - messaging/access-list/add
  - push/wake
---

## Abstract

The **Messaging — Wake Request** Trust Task lets a sender that has just queued mail for a DID ask that DID's **own mediator** to wake the device behind it. The mediator holds the device's opaque wake handle ([`messaging/wake/set`](../../set/0.1/spec.md)) and alone decides whether to ring; when it does, it sends [`push/wake`](../../../../push/wake/0.2/spec.md) to the device's push gateway as the device's **queue-driven trigger** ([push wake-up binding §3.4](../../../../../bindings/push/0.1/spec.md#34-triggering-a-wake)).

The motivating case is a named approver. A VTA holds a sensitive task for consent and pushes the signed request to a family member's phone, which has no account and no standing at that VTA — on purpose. The request then waits in the phone's mediator queue until the phone is next opened, and the window for the decision often runs out first. The VTA cannot ring the phone itself: it holds no handle, and it should not. With this task it names the message it just queued and asks the phone's mediator to ring.

Three properties hold whatever the sender asks:

- **Contentless.** The request names a DID and message identifiers. It carries no task type, reason, relying party or content, and the push it causes carries only the binding's contentless fields.
- **Bound to queued mail.** The mediator rings only for messages that are queued for the addressee right now, that the requester could name only because it held them, and that it has not rung for before.
- **Silent.** The answer is always `{}`, so a sender learns nothing about whether the addressee exists, is wakeable, is online, or was rung.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer

A conforming **producer** (the sender) **MUST**:

1. Address the request to the mediator the addressee's DID document names for message delivery, with itself as `issuer`, an `issuedAt`, and a `proof` whose `proofPurpose` is `authentication`.
2. Name in `msgIds` only messages it queued for the addressee itself, by the identifier the addressee's mediator stores each under — for a DIDComm message, the identifier derived from the packed message that mediator stores (the inner message a `forward` carried, where the sender's own mediator relayed it), as [`messaging/message/status`](../../../message/status/0.2/spec.md) defines it.
3. Send at most one request per message it wants rung, after the message has been sent. A producer **MUST NOT** re-send a request for a message whose earlier request it made, including when it later asks itself whether the addressee answered: a pending request is not re-rung.
4. Treat the request as fire-and-forget. It **MUST NOT** infer anything from the response, from its absence, or from how long either took, and **MUST NOT** retry on silence.
5. Carry nothing else. It **MUST NOT** carry a task type, a reason, the name of a relying party, or anything derived from the queued message in `ext` or anywhere else in the document.

### Consumer

A conforming **consumer** (the mediator) **MUST** validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and take the verified `issuer` as the requester, *W*. Where a session also authenticates the sender, that identity **MUST** equal *W*; otherwise the request is refused with `identityMismatch` as §7.2 requires.

It then rings the addressee's device **only if every one** of the following holds, and otherwise does nothing:

1. **A wakeable account.** `addressee` is a local account with a wake handle set ([`messaging/wake/set`](../../set/0.1/spec.md)).
2. **Queued, fresh mail.** Every identifier in `msgIds` names a message queued for `addressee`, not yet delivered to it, and stored within the last 10 minutes. A message delivered over a live stream or by pickup no longer qualifies.
3. **Mail from the requester.** Where a named message's stored envelope reveals its sender — the `skid` of a DIDComm authcrypt message — that sender is *W*. Where the stored envelope does not reveal its sender (a nested or sealed envelope), holding the identifier is the binding: it can be computed only by a party that held the stored bytes.
4. **Not blocked.** *W* is not refused by the addressee's [access list](../../../access-list/add/0.1/spec.md).
5. **Not online.** The addressee has no live delivery connection. Where it has one, the mail is streamed to it instead, and nothing rings.
6. **Not rung before.** No named message has already caused a ring. Each message rings at most once, however many requests name it and whoever sends them.
7. **Within limits.** The request fits a per-(*W*, `addressee`) budget and a per-`addressee` budget. A mediator **SHOULD** default to at most one ring per minute and ten per day for a (*W*, `addressee`) pair, and to coalescing rings for one addressee into at most one per 15 seconds and 30 per hour, and **SHOULD** make each configurable.

**Arming.** A request whose messages are not yet stored — the sender's relay is slower than its wake request — **MAY** be held for up to 120 seconds and evaluated again when a named message is stored. A request still unmet after that is discarded.

**Ringing.** To ring, the mediator sends [`push/wake/0.2`](../../../../push/wake/0.2/spec.md) as itself to the gateway named in the account's handle, with the handle, `v: 1`, `mediator` set to its own DID, and `urgency` copied from the request when present — and nothing else. It reaches the gateway over the highest-preference transport both parties advertise. On `push/wake:tokenUnregistered` it drops the account's handle; on `push/wake:notAllowed` it stops ringing that handle until the account sets one again.

**The answer.** The mediator **MUST** answer every request it accepted for processing with the same `{}` response, at the same point — before evaluating rules 1 to 7 or after, but in the same way for every outcome. It **MUST NOT** answer with a `trust-task-error` that depends on any of rules 1 to 7, the arming, or the outcome of the ring. Errors it raises before rule 1 — a malformed document, a proof that does not verify, a rate limit applied to *W* alone before any addressee is looked up — reveal nothing about the addressee and are answered as usual.

**Receiving the request.** A mediator that serves this task **MUST** accept it addressed to itself from a sender that holds no account there — over a session relayed by another mediator, or an unauthenticated one — and **MUST NOT** create an account for that sender or accept any other task type from it on the strength of this one. It **SHOULD** apply its inbound rate limit to such senders.

## Authorization

The authority is **having mail queued for the addressee**. Any sender may ask, and the mediator grants nothing on who the sender is: it rings only where the sender names messages it queued for the addressee, fresh and undelivered (rules 2 and 3), that the addressee has not refused (rule 4). The `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)) so that rules 3 and 7 can be applied to that party; it is not itself an authorization.

**Holding a handle grants no authority to wake.** The requester never holds the addressee's handle, and this task gives it none. The mediator acts on the handle as the device's trigger; the gateway still rings only for a trigger on the handle's allowlist, which the handle's controller set. A sender that somehow held a handle could not use this task to ring it, because the task names a DID and queued mail, never a handle.

**What the ring authorizes is nothing.** A wake carries no content and causes no action. The device connects, authenticates to its mediator, and collects its queue as it would on opening; every message it then processes is authenticated and authorized under its own task's rules. A request for consent that the wake made visible sooner is still decided by the person, on the request's own terms.

## Definitions

- **`addressee`** — REQUIRED. The DID the queued messages are addressed to, which is an account at this mediator.
- **`msgIds`** — REQUIRED. One to eight distinct identifiers of messages the requester queued for the addressee, as the addressee's mediator stores them. Each is a capability only a party that held the stored message can produce.
- **`urgency`** — OPTIONAL. `interactive` or `background`, passed through as [`push/wake`](../../../../push/wake/0.2/spec.md)'s hint. A producer handling a decision a person is waiting on **MAY** send `interactive`; it **MUST NOT** choose the value by task type in a way that lets the gateway or push provider tell kinds of task apart beyond that one bit.

## Request

The sender sends the request to the addressee's mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A VTA asks the approver's mediator to ring for the consent request it just pushed

```json
{
  "id": "urn:uuid:8c3f2a71-6d4b-4e19-b0a5-2f7d9e1c4a60",
  "type": "https://trusttasks.org/spec/messaging/wake/request/0.1",
  "issuer": "did:webvh:QmSam:vta.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-14T09:12:00Z",
  "payload": {
    "addressee": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
    "msgIds": [
      "5e3f7c2a8b10d4a7c3e5f8a1b2d4c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8"
    ],
    "urgency": "interactive"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmSam:vta.example#key-1",
    "created": "2026-11-14T09:12:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3mT..."
  }
}
```

## Response

The mediator answers with an empty payload, the sub-schema reachable via `$anchor: "response"`, whatever it did. Errors raised before the addressee is considered use `trust-task-error` as usual.

### The same answer for every outcome

```json
{
  "id": "urn:uuid:8c3f2a71-6d4b-4e19-b0a5-2f7d9e1c4a61",
  "type": "https://trusttasks.org/spec/messaging/wake/request/0.1#response",
  "threadId": "urn:uuid:8c3f2a71-6d4b-4e19-b0a5-2f7d9e1c4a60",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmSam:vta.example",
  "issuedAt": "2026-11-14T09:12:00Z",
  "payload": {}
}
```

A sender cannot tell from this whether the phone was rung, was already online, has no handle, has blocked it, or is not an account here. That is the design: a per-outcome answer would let anyone holding a DID ask the mediator whether that DID is registered, wakeable or online. Whether rings happen is an operator's question, answered by the mediator's own metrics, not by the protocol.

## Security & Privacy

### Data carried

The request carries the addressee's DID — which the sender already used to address mail — up to eight message identifiers, and an optional one-bit urgency hint. No message content, task type, reason or relying party travels, and the push it may cause carries only the [push binding §2](../../../../../bindings/push/0.1/spec.md#2-what-this-binding-carries--and-what-it-must-not) fields. The response carries nothing. A producer **MUST NOT** carry anything else in `ext`; a mediator ignores `ext` when deciding.

### Correlation

The mediator learns that *W* sent mail to the addressee, which it largely knew from storing that mail. It learns nothing new about the message. The gateway and the push provider see a wake from the mediator, as they would for any queue-driven wake, and never learn *W*: the mediator rings as itself.

The response is built to close the obvious oracle. Because every accepted request is answered `{}` in the same way, a sender holding a DID cannot use this task to learn whether that DID is an account here, has a wake handle, is online, has blocked it, or was rung. The remaining channel is the device's behaviour — a phone that answers its consent request within seconds was evidently woken — and that is information the sender's own exchange with the device reveals whatever this task does.

The mediator declares `identifierScope: public` because senders find it the way they find it for mail: from the service the addressee's DID document advertises. A request must reach the very mediator that stores the addressee's queue, under the DID that its stored message identifiers belong to. The sender declares `any`; the mediator compares it only with the sender of the mail it holds.

### Retention

The mediator keeps an armed request for at most 120 seconds, and a marker per message that it has rung, for as long as the message is queued, so rule 6 can hold. It keeps no record of the request beyond those and the rate-limit counters of rule 7, and **SHOULD** keep its metrics aggregate (`wake requests by outcome`) rather than per sender or per addressee.

### Consent/purpose

The purpose is to bring the addressee to mail it already has, sooner. A mediator that rang without queued mail, rang for a party the addressee has refused, or rang more often than the limits allow would turn the doorbell into a way for a stranger to buzz someone's phone at will; rules 2 to 7 exist to prevent that, and the addressee's access list is how it refuses a sender outright. The ring itself decides nothing: a person who is woken still reads and decides on the request under that request's own rules.
