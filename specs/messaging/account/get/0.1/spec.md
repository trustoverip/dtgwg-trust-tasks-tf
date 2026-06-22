---
slug: messaging/account/get
version: "0.1"
title: Messaging — Get Account
summary: A requester fetches the mediator's view of one served account — its role, access-control capabilities, queue limits, and current queue state.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - read
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
  requirement: RECOMMENDED
  rationale: Reading an account is a non-mutating query; a proof is RECOMMENDED to bind the request to its requester for authorization and audit, but is not required for integrity of any change since none is made.
errorCodes:
  - code: messaging/account/get:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
related:
  - messaging/account/add
  - messaging/account/list
  - messaging/acl/get
---

## Abstract

The **Messaging — Get Account** Trust Task fetches the mediator's view of a single served account. The requester names the target [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid); the mediator returns the full [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) — its [`AccountType`](../../../_shared/0.1/messaging.schema.json#/$defs/AccountType), its realized [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) capabilities, its [`QueueLimits`](../../../_shared/0.1/messaging.schema.json#/$defs/QueueLimits), and its current queue state.

The task is read-only; it makes no change to the account. The mediator applies its own authorization policy — typically an administrator may read any account and a `standard` account may read its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/get/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID.
3. **SHOULD** include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and, where a `proof` is present, verify it.
2. Enforce its own authorization policy and respond with the framework's `permissionDenied` where the requester may not read the target account.
3. Where the target DID has no account, respond with `messaging/account/get:unknownAccount`.
4. Return the full [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) view in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/get/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/get/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example"
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

This example fetches the mediator's view of Alice's account.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/get/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full mediator view of the requested account.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/get/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "account": {
      "did": "did:web:alice.example",
      "accountType": "standard",
      "acl": {
        "accessListMode": "explicitDeny",
        "blocked": false,
        "local": true,
        "sendMessages": true,
        "receiveMessages": true,
        "sendForwarded": false,
        "receiveForwarded": true,
        "createInvites": false,
        "anonReceive": false,
        "selfManageList": false,
        "selfManageSendQueueLimit": false,
        "selfManageReceiveQueueLimit": false,
        "didcommEnabled": true,
        "tspEnabled": true
      },
      "queueLimits": {
        "sendQueueLimit": 1000,
        "receiveQueueLimit": 1000
      },
      "sendQueueCount": 3,
      "sendQueueBytes": 8192,
      "receiveQueueCount": 0,
      "receiveQueueBytes": 0,
      "accessListCount": 2
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

An account view discloses the account's role, capabilities, limits, and current queue depth — operational metadata about the served party. A mediator **MUST** enforce its own authorization independent of the document, returning the framework's `permissionDenied` where the requester may not read the target account, even though `proof` is only **RECOMMENDED** for this read-only task.

A `proof`, when present, binds the request to its requester for authorization and audit; the queue-state counts it returns are point-in-time and **MAY** be stale by the time the response is read.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
