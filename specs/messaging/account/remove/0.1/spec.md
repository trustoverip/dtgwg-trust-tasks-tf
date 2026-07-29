---
slug: messaging/account/remove
version: "0.1"
title: Messaging — Remove Account
summary: An administrator removes a served account from the mediator, refusing to remove protected mediator or rootAdmin accounts.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - deprovisioning
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
  rationale: Removing an account is a destructive administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: destructive
  rationale: "Removes a served account and its queue state; re-adding creates a fresh account, not a restoration."
consequences:
  - "Deletes the account together with its queued-message state."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/account/remove:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/account/remove:protectedAccount
    meaning: The target account is protected (a mediator or rootAdmin account) and cannot be removed.
    retryable: false
related:
  - messaging/account/add
  - messaging/account/get
  - messaging/account/update
---

## Abstract

The **Messaging — Remove Account** Trust Task removes a served account from the mediator. The *administrator* names the target [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid); the mediator deprovisions the account and confirms the removal.

The mediator **MUST** refuse to remove a protected account — its own `mediator` account or a `rootAdmin` account — responding with `messaging/account/remove:protectedAccount`. The mediator applies its own authorization policy; typically only an `admin`/`rootAdmin` may remove accounts.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/remove/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Enforce its own authorization policy and respond with the framework's `permissionDenied` where the requester may not remove accounts.
3. Where the target DID has no account, respond with `messaging/account/remove:unknownAccount`.
4. Where the target account is protected — the mediator's own `mediator` account or a `rootAdmin` account — refuse and respond with `messaging/account/remove:protectedAccount`.
5. Otherwise remove the account and return `{ did, removed: true }`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/remove/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/remove/0.1",
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

This example removes Alice's account from the mediator.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/remove/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, removed }`, confirming the account and the outcome.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/remove/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "removed": true
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Removing an account is a destructive administrative mutation that revokes standing and may strand queued messages: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the requester may remove the account.

The protected-account refusal guards the mediator against removing its own `mediator` account or a `rootAdmin` account and so locking out administration; a mediator **MUST NOT** remove a protected account regardless of requester standing. A removal is generally irreversible and discards the account's queue and access list, so a mediator **SHOULD** treat it as terminal.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
