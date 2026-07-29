---
slug: messaging/account/change-queue-limits
version: "0.1"
title: Messaging — Change Account Queue Limits
summary: An administrator sets a served account's send and receive queued-message limits, applying a partial update where -1 means unlimited and an omitted member leaves that limit unchanged.
status: retired
supersededBy: messaging/account/update
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - queue
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Changing an account's queue limits is an administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Partial update of an account's queue limits."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/account/change-queue-limits:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/account/change-queue-limits:selfChangeDenied
    meaning: A standard account that lacks the relevant selfManage*QueueLimit capability attempted to change its own queue limit.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        deniedLimits:
          type: array
          items: { type: string }
related:
  - messaging/account/change-type
  - messaging/account/get
  - messaging/acl/set
---

## Abstract

The **Messaging — Change Account Queue Limits** Trust Task sets a served account's queued-message limits. The *administrator* names the target [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid) and the [`QueueLimits`](../../../_shared/0.1/messaging.schema.json#/$defs/QueueLimits) to apply; the mediator updates the limits and returns the realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) view.

The update is **partial**: a [`QueueLimits`](../../../_shared/0.1/messaging.schema.json#/$defs/QueueLimits) member that is present is applied, a member that is omitted leaves that limit unchanged, and a value of `-1` sets the limit to unlimited. The mediator applies its own authorization policy: an `admin`/`rootAdmin` requester may set any account's limits, while a `standard` account may change only the limits it is permitted to self-manage (`selfManageSendQueueLimit` / `selfManageReceiveQueueLimit`).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by [`messaging/account/update`](../../update/0.1/spec.md), which accepts this task's exact payload — send `{ did, queueLimits }` with the same members and the same partial-update semantics (`-1` = unlimited, omitted member unchanged); the `selfChangeDenied` guard carries over per member.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/change-queue-limits/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.queueLimits` with the limit members to apply (`-1` = unlimited; an omitted member is left unchanged).
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/account/change-queue-limits:unknownAccount`.
3. Where the requester is the account's own `standard` controller and is changing a limit it is not permitted to self-manage, respond with `messaging/account/change-queue-limits:selfChangeDenied` listing the denied limits, or with the framework's `permissionDenied` where the requester has no standing at all.
4. Apply the present limit members as a partial update, treating `-1` as unlimited and leaving omitted members unchanged, and return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/change-queue-limits/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/change-queue-limits/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "queueLimits": {
      "sendQueueLimit": 5000,
      "receiveQueueLimit": -1
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-22T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

This example sets Alice's send-queue limit to 5000 messages and makes her receive queue unlimited (`-1`).

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/change-queue-limits/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full realized mediator view after the change.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/change-queue-limits/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "account": {
      "did": "did:web:alice.example",
      "accountType": "standard",
      "acl": {
        "blocked": false,
        "sendMessages": true,
        "receiveMessages": true,
        "didcommEnabled": true,
        "tspEnabled": true
      },
      "queueLimits": {
        "sendQueueLimit": 5000,
        "receiveQueueLimit": -1
      }
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Setting queue limits is an administrative mutation that governs how much undelivered traffic the mediator will buffer for an account: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the change is permitted.

An unlimited (`-1`) limit removes a back-pressure control and can let a single account exhaust mediator storage; a mediator **SHOULD** treat granting it as a privileged action and **MUST** restrict a `standard` account to changing only the limits it is permitted to self-manage. The mediator **SHOULD** return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) so the administrator can confirm the applied limits.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
