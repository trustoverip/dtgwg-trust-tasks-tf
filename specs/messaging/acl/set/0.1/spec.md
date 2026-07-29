---
slug: messaging/acl/set
version: "0.1"
title: Messaging — Set ACL
summary: An administrator sets the mediator's access-control capability flags for a served account, applying a partial update of named capabilities (send, receive, forward, anon-receive, protocol enablement, self-manage, blocked).
status: retired
supersededBy: messaging/account/update
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - acl
  - access-control
  - admin
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
  rationale: A capability change is an administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Partial update of an account's mediator capability flags; reversible."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/acl/set:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/acl/set:selfChangeDenied
    meaning: The requester is the account's own controller and attempted to change a capability whose self-management is not permitted for it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        deniedCapabilities:
          type: array
          items: { type: string }
related:
  - messaging/acl/get
  - messaging/account/get
  - messaging/access-list/add
---

## Abstract

The **Messaging — Set ACL** Trust Task sets the mediator's per-account access-control capabilities for a served account. The *administrator* declares the [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) capability flags the mediator should hold for the target DID. The update is **partial**: only the capability members present in `payload.acl` are applied; members omitted are left unchanged. The mediator applies its own authorization policy — an `admin`/`rootAdmin` requester may set any account's capabilities; a `standard` account may change only the capabilities it is permitted to self-manage (`selfManage*`).

The mediator's internal capability representation (a packed flag set) is opaque to the framework; this task carries the capabilities as the named-boolean [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) object so the document is human-meaningful and transport-agnostic.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by [`messaging/account/update`](../../../account/update/0.1/spec.md), which accepts this task's exact payload — send `{ did, acl }` with the same partial-update semantics (a capability present is set, a capability omitted is unchanged) and the same `selfChangeDenied` self-management guard. The successor returns the full realized `Account` rather than `{ did, acl }`; the applied capability set is its `account.acl` member.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/acl/set/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.acl` with the capability members to apply.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/acl/set:unknownAccount`.
3. Where the requester is not an administrator and is changing a capability it is not permitted to self-manage, respond with `messaging/acl/set:selfChangeDenied` listing the denied capabilities, or with the framework's `permissionDenied` where the requester has no standing at all.
4. Apply the present capability members as a partial update, leave omitted members unchanged, and return the full realized [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/acl/set/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/acl/set/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "acl": {
      "sendMessages": true,
      "receiveMessages": true,
      "sendForwarded": true,
      "tspEnabled": false
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

This example enables messaging and forwarding for Alice and marks her account DIDComm-only (`tspEnabled: false`); every other capability is left unchanged.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/acl/set/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, acl }`, where `acl` is the full realized capability set the mediator now holds for the account.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/acl/set/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "acl": {
      "accessListMode": "explicitDeny",
      "blocked": false,
      "local": true,
      "sendMessages": true,
      "receiveMessages": true,
      "sendForwarded": true,
      "receiveForwarded": true,
      "createInvites": false,
      "anonReceive": false,
      "selfManageList": false,
      "selfManageSendQueueLimit": false,
      "selfManageReceiveQueueLimit": false,
      "didcommEnabled": true,
      "tspEnabled": false
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A set-ACL document is an administrative mutation: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the change is permitted.

The `tspEnabled` / `didcommEnabled` flags govern which transport a recipient is served over; disabling a protocol an account still relies on will silently strip its delivery, so a mediator **SHOULD** reflect the realized flags in the response for the administrator to confirm.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
