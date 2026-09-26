---
slug: messaging/message/delete
version: "0.1"
title: "Messaging — Delete Messages"
summary: "Delete up to 100 named messages from one account's mediator queues, with a per-message result — the owner deleting its own, or an administrator clearing another account's."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Requester (the account owner, or an administrator)
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Deleting a queued message is irreversible and, for an administrator acting on another account, removes mail that account never received; the request must be attributable after the fact so the loss can be traced to whoever caused it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A destructive task is consequential (SPEC §7.3 item 17); a replayed delete must be placeable in time so it falls outside the acceptance window rather than re-running against a queue whose contents have since changed.
sideEffects:
  level: destructive
  rationale: >-
    Removes the named messages from the mediator's store. An undelivered message deleted here is gone; it is not returned to its sender.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
retention:
  class: durable
  rationale: >-
    A deletion on another account's behalf is an administrative act the mediator records in its audit log; the record is what lets an operator answer "where did my message go".
errorCodes:
  - code: messaging/message/delete:unknownAccount
    meaning: The named account (`did`) has no account at this mediator.
    retryable: false
  - code: messaging/message/delete:rootAdminRequired
    meaning: The target account holds the admin, rootAdmin, or mediator role, and the requester is not a rootAdmin.
    retryable: false
related:
  - messaging/message/list
  - messaging/message/get
  - messaging/queue/purge
---

## Abstract

The **Messaging — Delete Messages** Trust Task removes named messages from one account's mediator queues. It is the surgical counterpart to [`messaging/queue/purge`](../../../queue/purge/0.1/spec.md). A purge clears messages by selection (a whole queue, a counterparty, an age). This task clears messages by `msgId`, typically the ones an operator has just inspected with [`messaging/message/list`](../../list/0.1/spec.md) or [`messaging/message/get`](../../get/0.1/spec.md).

The result is **per message** and in request order. A message that could not be deleted is reported as such with a reason; it never fails the whole task, and it is never reported as deleted.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the requester) **MUST**:

1. Emit a document whose `type` is `https://trusttasks.org/spec/messaging/message/delete/0.1`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, and a `proof`.
2. Omit `did` to delete from its own account, or set it to the account whose queues hold the messages. Name between 1 and 100 distinct `msgIds`.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Resolve the target account: `did` when present, otherwise the requester. Where it has no account, respond `messaging/message/delete:unknownAccount`.
3. Apply the [Authorization](#authorization) rules below, failing the whole task (`permissionDenied` or `rootAdminRequired`) before deleting anything.
4. For each `msgId`, in request order, return exactly one result:
   - `deleted: true` only when this request removed the message from the target account's queues.
   - `deleted: false, reason: notFound` when no such message exists (including one already deleted or expired).
   - `deleted: false, reason: notInAccount` when the message exists but is held against a different account.
5. **Never** report `deleted: true` for a message it did not remove. This includes a message whose removal it attempted but whose outcome it cannot confirm: such an item **MUST** be reported `deleted: false`, or the whole task **MUST** fail with `taskFailed`.
6. Record an audit entry naming the requester, the target account, and the removed `msgId`s whenever the target account is not the requester's own.

## Authorization

The authority is **ownership of the queue**, extended to administrators.

- **Own queues.** Any account may delete messages from its own queues, under the same capability gate the mediator applies to that account's own message deletion. For a mediator that exposes the `local` capability, that means the account must hold it. Otherwise the response is `permissionDenied`.
- **Another account's queues.** An `admin` or `rootAdmin` may delete from a `standard` account's queues. Deleting from a privileged account's queues (one holding the `admin`, `rootAdmin`, or `mediator` role) requires a `rootAdmin` (`rootAdminRequired`). This prevents an admin from erasing traffic addressed to the operators who oversee it. A `standard` requester naming any other account receives `permissionDenied`.

The `proof` establishes who asked, and makes the audit record non-repudiable. It does not establish that they may. The mediator decides entitlement from its own account records.

## Definitions

- **`did`** — OPTIONAL. The account whose queues hold the messages, as a [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid). Omitted means the requester's own account.
- **`msgIds`** — REQUIRED. 1–100 distinct message identifiers, as reported in `MessageMeta.msgId`. One request addresses one account. Messages from several accounts take several requests.
- **`results`** (response) — one entry per requested id, in request order: `msgId`, `deleted`, and, when `deleted` is false, a `reason` of `notFound` or `notInAccount`.

## Request

The requester sends the request to its mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator removes two messages from Alice's queue

```json
{
  "id": "urn:uuid:2b8e4a60-51c7-4a9e-8f0d-6b3c1d2e4f01",
  "type": "https://trusttasks.org/spec/messaging/message/delete/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:05:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "msgIds": ["1726912345000-0", "1726912399000-0"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4hq..."
  }
}
```

## Response

The mediator responds with one result per requested id, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures of the task as a whole use `trust-task-error`. Failures of individual ids are results, not errors.

### One deleted, one already gone

```json
{
  "id": "urn:uuid:2b8e4a60-51c7-4a9e-8f0d-6b3c1d2e4f02",
  "type": "https://trusttasks.org/spec/messaging/message/delete/0.1#response",
  "threadId": "urn:uuid:2b8e4a60-51c7-4a9e-8f0d-6b3c1d2e4f01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:05:01Z",
  "payload": {
    "results": [
      { "msgId": "1726912345000-0", "deleted": true },
      { "msgId": "1726912399000-0", "deleted": false, "reason": "notFound" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries an account identifier and message handles. The response carries only those handles and their outcomes. No message content moves in either direction, and a producer **MUST NOT** place message content or counterparty details in `ext`.

### Correlation

The `msgIds` link this request to the `messaging/message/list`, `messaging/message/get` or `messaging/monitor/event` documents they were taken from. This is inherent to deleting by handle. `notInAccount` tells the requester that a message exists under another account. Only an administrator can reach that answer for an account other than its own, and it reveals existence only, never the holder.

### Retention

The mediator **MUST** keep the audit record of a cross-account deletion for as long as it retains its other administrative audit entries. It is the only remaining trace of mail that was never delivered. An own-account deletion needs no record beyond the mediator's ordinary operational logs.

### Consent/purpose

The purpose is to clear messages that are stuck, unwanted, or blocking a queue: the owner managing its own mail, or an operator restoring service to an account. Whether an administrator's deletion of another account's messages needs that account's agreement, or anyone else's, is the mediator operator's policy, not this specification's.
