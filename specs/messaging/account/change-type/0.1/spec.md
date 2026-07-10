---
slug: messaging/account/change-type
version: "0.1"
title: Messaging — Change Account Type
summary: An administrator changes a served account's role, with assignment or modification of the rootAdmin role reserved to a rootAdmin.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - role
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
  rationale: Changing an account's role is a privilege-granting administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Changes a served account's role; reversible."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/account/change-type:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/account/change-type:rootAdminRequired
    meaning: Assigning or modifying the rootAdmin role requires the requester to be a rootAdmin.
    retryable: false
related:
  - messaging/account/add
  - messaging/account/change-queue-limits
  - messaging/account/get
---

## Abstract

The **Messaging — Change Account Type** Trust Task changes the role of a served account. The *administrator* names the target [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid) and the [`AccountType`](../../../_shared/0.1/messaging.schema.json#/$defs/AccountType) to assign; the mediator updates the role and returns the realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) view.

Changing a role is a privilege grant. The mediator applies its own authorization policy: assigning or modifying the `rootAdmin` role — whether promoting an account to `rootAdmin` or changing an account that is currently `rootAdmin` — is reserved to a `rootAdmin` requester and otherwise responds with `messaging/account/change-type:rootAdminRequired`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/change-type/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.accountType` with the role to assign.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Enforce its own authorization policy and respond with the framework's `permissionDenied` where the requester may not administer the target account.
3. Where the target DID has no account, respond with `messaging/account/change-type:unknownAccount`.
4. Where the request would assign the `rootAdmin` role, or would change an account that is currently `rootAdmin`, and the requester is not a `rootAdmin`, respond with `messaging/account/change-type:rootAdminRequired`.
5. Otherwise apply the new role and return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/change-type/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/change-type/0.1",
  "issuer": "did:web:root.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "accountType": "admin"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:root.example#key-1",
    "created": "2026-06-22T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

This example promotes Alice's account to the `admin` role.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/change-type/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full realized mediator view after the role change.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/change-type/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:root.example",
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
      }
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Changing an account's role is a privilege-granting administrative mutation: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the requester may grant the requested role.

The `rootAdmin` role is the mediator's highest privilege; a mediator **MUST** restrict both assigning it and modifying an account that already holds it to a `rootAdmin` requester, so an `admin` cannot escalate itself or others to `rootAdmin` nor demote the standing root. The mediator **SHOULD** return the full realized [`Account`](../../../_shared/0.1/messaging.schema.json#/$defs/Account) so the administrator can confirm the new role.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
