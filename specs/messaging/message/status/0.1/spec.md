---
slug: messaging/message/status
version: "0.1"
title: "Messaging — Message Status"
summary: "A sender asks its mediator where the messages it sent now stand — still queued, handed over, collected by the recipient, withdrawn, or discarded — so that delivery can be confirmed on evidence rather than inferred from an outbox entry disappearing."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Sender (the account that sent the messages)
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The answer is bound to who asks: a mediator answers only for messages the requester sent, so the request's proof is what establishes the requester independent of the transport. The response is RECOMMENDED so a sender can retain it as evidence that a message was collected, which is the purpose of the task.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on these requests. Without issuedAt a captured request could be replayed to watch, over time, when a sender's recipients collect their messages.
sideEffects:
  level: none
  rationale: >-
    A read of stored state and receipts. Asking does not deliver, mark, extend, or delete any message or receipt.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    Returns, for each message the requester sent, a state and optionally the time it was reached. The request carries message identifiers. No message body travels in either direction.
retention:
  class: transient
  rationale: >-
    The mediator keeps nothing of the request. The receipts it answers from are created by the deletion of a message, not by this task, and are kept only for the bounded period described under Retention.
related:
  - messaging/message/list
  - messaging/message/get
  - messaging/message/delete
  - messaging/queue/status
---

## Abstract

The **Messaging — Message Status** Trust Task lets a sender ask its mediator where the messages it sent now stand. For each message identifier the mediator answers one [`MessageState`](payload.schema.json): still `queued`, `delivered` to the recipient but not yet acknowledged, `collected` by the recipient, `withdrawn` by the sender, `discarded` by the mediator, or `unknown`.

A sender otherwise has only one signal that a message arrived: it sees the message in its send queue ([`messaging/message/list`](../../list/0.1/spec.md)), and later it is gone. That signal is wrong in both directions. A recipient connected live typically collects within milliseconds, before any poll has seen the message queued, so a collected message produces no evidence at all. And a message leaves the send queue in the same way whether the recipient took it, the mediator expired it, or its account was removed, so "gone" cannot tell collection from loss. A sender that retries on missing evidence sends collected messages again; a sender that treats absence as delivery records lost messages as delivered.

This task closes the gap. A conforming mediator records, when a message the sender could see leaves its queue, a short-lived **receipt** saying why it left, and answers from it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the sender) **MUST** emit a document whose `type` is `https://trusttasks.org/spec/messaging/message/status/0.1`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, and a `proof`, naming in `msgIds` the identifiers of messages it sent.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Answer with exactly one entry per requested `msgId`, in request order.
3. Answer only from the requester's own sending. A message is the requester's when the mediator recorded the requester as its sender. For any other identifier — another account's message, a message sent anonymously, a message the requester received rather than sent, or one the mediator never held — answer `unknown`, identically, so that the response cannot be used to learn whether a message exists.
4. For a message it still holds, answer `queued` when it has not been handed to the recipient, and `delivered`, with `at` set to the first hand-over, when it has but the recipient has not removed it.
5. For a message that has left the queue, answer from the receipt recorded when it left (see [Receipts](#receipts)), or `unknown` when no receipt remains.
6. **Not** change any message or receipt in answering: not deliver, not mark, not extend an expiry, not delete.

## Receipts

When a message whose sender is known leaves the queue, a conforming mediator **MUST** record a receipt for that sender, keyed by the message identifier, carrying why it left and when:

- **`collected`** — the recipient removed it: it acknowledged the message after pickup, fetched it with delete, or deleted or purged it. The recipient took responsibility for the message; whether it went on to read it is not the mediator's to know.
- **`withdrawn`** — the sender removed it before the recipient did.
- **`discarded`** — the mediator removed it before the recipient did: it expired, or the recipient's account was removed, or an administrator removed it.

The reason is established by **who removed the message**, not by the path the removal took. A mediator **MUST NOT** record `collected` for a removal the recipient did not make. That is the property the whole task rests on: a sender that settles delivery on `collected` must never be told a lost message arrived.

A mediator **MUST** record a receipt only for messages whose sender it recorded, and **MUST** key it under that sender, so no other account can ever read it.

A receipt is kept for a bounded period. A mediator **SHOULD** keep receipts for at least 24 hours, so that a sender polling on a delivery window measured in minutes or hours reliably finds them, and **SHOULD** make the period configurable. After it, the answer for that message is `unknown`. A sender **MUST** therefore treat `unknown` as "no evidence", never as either delivery or loss.

A message that never entered the requester's send queue — one forwarded to another mediator, or sent without an authenticated sender — has no receipt at this mediator, and its status is always `unknown` here.

## Authorization

The authority is **being the message's sender**. Any account may ask about the messages it sent, and only about those. The mediator **SHOULD** apply the same capability gate it applies to the account's own queue operations; for a mediator that exposes the `local` capability, a requester without it receives `permissionDenied`.

There is no administrative form. An administrator that needs to see another account's traffic uses [`messaging/message/list`](../../list/0.1/spec.md). This task answers what a sender is entitled to know about its own messages, and an administrator's view of a queue is a different question with a different audit.

The `proof` establishes **who** asked. The mediator derives the sender of each message from its own records, never from the document.

## Definitions

- **`msgIds`** — REQUIRED. One to 100 distinct identifiers of messages the requester sent: the mediator's identifier for each stored message, as returned in `MessageMeta.msgId` by `messaging/message/list`. For a DIDComm message the mediator derives it from the stored message itself, so a sender can compute it from what it sent.
- **`statuses`** (response) — one entry per requested identifier, in request order.
- **`state`** (response) — the [`MessageState`](payload.schema.json) of that message: `queued`, `delivered`, `collected`, `withdrawn`, `discarded`, or `unknown`.
- **`at`** (response) — when the message reached that state. Present for `delivered`, `collected`, `withdrawn` and `discarded` when the mediator discloses it; absent otherwise.

## Request

The sender sends the request to its mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A sender asks about three messages it pushed a minute ago

```json
{
  "id": "urn:uuid:0b6d3f4a-2c81-4e57-9d1a-5e3f7c2a8b10",
  "type": "https://trusttasks.org/spec/messaging/message/status/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-26T09:30:00Z",
  "payload": {
    "msgIds": [
      "9f2c1e6b0d4a7c3e5f8a1b2d4c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e",
      "4a7c3e5f8a1b2d4c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e9f2c1e6b0d",
      "c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e9f2c1e6b0d4a7c3e5f8a1b2d4"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:community.example#key-1",
    "created": "2026-09-26T09:30:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4jK..."
  }
}
```

## Response

The mediator responds with one status per message, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### One collected, one still with a recipient that is offline, one expired

```json
{
  "id": "urn:uuid:0b6d3f4a-2c81-4e57-9d1a-5e3f7c2a8b11",
  "type": "https://trusttasks.org/spec/messaging/message/status/0.1#response",
  "threadId": "urn:uuid:0b6d3f4a-2c81-4e57-9d1a-5e3f7c2a8b10",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-09-26T09:30:01Z",
  "payload": {
    "statuses": [
      {
        "msgId": "9f2c1e6b0d4a7c3e5f8a1b2d4c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e",
        "state": "collected",
        "at": "2026-09-26T09:29:02Z"
      },
      {
        "msgId": "4a7c3e5f8a1b2d4c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e9f2c1e6b0d",
        "state": "queued"
      },
      {
        "msgId": "c6e8f0a1b3d5c7e9f1a2b4c6d8e0f2a4b6c8d0e9f2c1e6b0d4a7c3e5f8a1b2d4",
        "state": "discarded",
        "at": "2026-09-26T09:15:00Z"
      }
    ]
  }
}
```

The first message was collected within a second of being sent, too fast for any poll of the send queue to have seen it. The third was discarded before its recipient took it, so a sender that inferred delivery from its disappearance would have recorded it as delivered.

## Security & Privacy

### Data carried

The request carries message identifiers the requester already holds. The response carries a state and, optionally, a time for each. No message body, counterparty, or size travels in either direction.

### Correlation

A `collected` or `delivered` time tells a sender when its recipient came online and took its messages. That is inherent in confirming delivery, and a sender could already approximate it by watching its send queue drain; this task makes it exact. A mediator **MAY** coarsen `at` (to the minute, say) or omit it, and the states remain correct without it. A deployment where recipients' activity times are themselves sensitive should do so.

Because every answer that is not about the requester's own sending is `unknown`, and `unknown` is also the answer after a receipt expires, the response cannot be used to probe for other accounts' messages.

### Retention

The mediator retains nothing of the request. Receipts are created by a message leaving the queue, are keyed under the message's sender, and are removed at the end of the bounded period described in [Receipts](#receipts). They carry the message identifier, the reason, and the time, and nothing else.

### Consent/purpose

The purpose is delivery assurance: letting a sender confirm on evidence that a message was collected, so it neither loses messages it believes delivered nor re-sends messages already collected. Using receipts to profile when a recipient is active is outside this task's purpose, and the coarsening permitted above exists for deployments that need to prevent it.
