---
slug: messaging/account/update
version: "0.1"
title: Messaging — Update Account
summary: An administrator applies a partial update to a served account — role, access-control capabilities, and queue limits in one task; a member omitted leaves that facet of the account unchanged.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - update
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
  rationale: Updating an account is an administrative mutation — and, when it assigns a role, a privilege grant — whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Partial update of a served account's role, capabilities, and queue limits; reversible."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/account/update:unknownAccount
    meaning: The target DID has no account at this mediator. This task updates; use messaging/account/add to create.
    retryable: false
  - code: messaging/account/update:rootAdminRequired
    meaning: Assigning the rootAdmin role, or updating an account that currently holds it, requires the requester to be a rootAdmin.
    retryable: false
  - code: messaging/account/update:selfChangeDenied
    meaning: The requester is the account's own controller and attempted to change a member it is not permitted to self-manage.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        deniedMembers:
          type: array
          items: { type: string }
related:
  - messaging/account/add
  - messaging/account/get
  - messaging/account/list
  - messaging/access-list/update
---

## Abstract

The **Messaging — Update Account** Trust Task applies a **partial update** to a served account. The *administrator* names the target [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid) and any combination of the account's mutable facets — its [`AccountType`](../../../_shared/0.1/messaging.schema.json#/$defs/AccountType) role, its [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) capabilities, and its [`QueueLimits`](../../../_shared/0.1/messaging.schema.json#/$defs/QueueLimits). The mediator applies the present members and returns the realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) view. The payload mirrors [`messaging/account/add`](../../add/0.1/spec.md) exactly: what `add` accepts at creation, `update` amends afterwards.

The update is **partial at every level**: a top-level member omitted leaves that facet unchanged, and within `acl` and `queueLimits` a member omitted leaves that capability or limit unchanged (`-1` sets a limit to unlimited). The authorization guards of the tasks this one replaces apply **per member**:

- `accountType` — changing a role is a privilege grant. Assigning `rootAdmin`, or updating any facet of an account that currently holds `rootAdmin`, is reserved to a `rootAdmin` requester (`rootAdminRequired`). Assigning any non-standard role requires administrative standing.
- `acl` — an `admin`/`rootAdmin` requester may set any account's capabilities; a `standard` account may change only the capabilities it is permitted to self-manage (`selfChangeDenied`).
- `queueLimits` — an `admin`/`rootAdmin` requester may set any account's limits; a `standard` account may change only the limits it self-manages (`selfManageSendQueueLimit` / `selfManageReceiveQueueLimit`).

This task supersedes `messaging/account/change-type`, `messaging/account/change-queue-limits`, and `messaging/acl/set`, which were three single-facet partial updaters of the same Account object.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/update/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and at least one of `payload.accountType`, `payload.acl`, `payload.queueLimits` with the members to apply; omit every member that should stay as it is.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/account/update:unknownAccount` — this task does not create.
3. Where the request would assign the `rootAdmin` role, or would change an account that currently holds it, and the requester is not a `rootAdmin`, respond with `messaging/account/update:rootAdminRequired`. Where `accountType` assigns any non-standard role and the requester lacks administrative standing, respond with the framework's `permissionDenied`.
4. Where the requester is the account's own `standard` controller and a present `acl` or `queueLimits` member is one it is not permitted to self-manage, respond with `messaging/account/update:selfChangeDenied` listing the denied members, or with the framework's `permissionDenied` where the requester has no standing at all.
5. Apply the present members as a partial update — leaving every omitted member unchanged, treating `-1` as unlimited in `queueLimits` — atomically: either every present member is applied or none is. Return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/update/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/update/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "accountType": "admin",
    "queueLimits": {
      "sendQueueLimit": 5000
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

This example promotes Alice to the `admin` role and raises her send-queue limit to 5000 in one update; her receive-queue limit and every capability flag are left unchanged.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/update/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full realized mediator view after the update.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/update/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "account": {
      "did": "did:web:alice.example",
      "accountType": "admin",
      "acl": {
        "blocked": false,
        "sendMessages": true,
        "receiveMessages": true,
        "didcommEnabled": true,
        "tspEnabled": true
      },
      "queueLimits": {
        "sendQueueLimit": 5000,
        "receiveQueueLimit": 1000
      }
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Updating an account is an administrative mutation, and when `accountType` is present it is a privilege grant: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the requester may apply the requested members.

The proof requirement is uniform across the payload's members deliberately. This task's predecessors disagreed — the role grant via `admin/add`/`admin/strip` was proof-**RECOMMENDED** while the identical grant via `account/change-type` was proof-**REQUIRED** — leaving the evidentiary standard of a privilege change dependent on which task the caller happened to pick. Here every path to the same mutation carries the same requirement.

The `rootAdmin` role is the mediator's highest privilege; a mediator **MUST** restrict both assigning it and updating an account that already holds it to a `rootAdmin` requester, so an `admin` can neither escalate to `rootAdmin` nor alter the standing root. An unlimited (`-1`) queue limit removes a back-pressure control; a mediator **SHOULD** treat granting it as a privileged action. The mediator **SHOULD** return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) so the administrator can confirm exactly what was applied.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
