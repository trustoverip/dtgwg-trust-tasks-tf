---
slug: messaging/access-list/remove
version: "0.1"
title: Messaging — Remove from Access List
summary: An administrator removes one or more DIDs from a served account's access list — the per-account set of other DIDs that, combined with the account's accessListMode, governs who may send to that account.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - access-list
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
  rationale: Removing entries from an account's access list is an administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
errorCodes:
  - code: messaging/access-list/remove:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/access-list/remove:selfChangeDenied
    meaning: The requester is the account's own controller and the account is not permitted to self-manage its access list (selfManageList is not set).
    retryable: false
related:
  - messaging/access-list/add
  - messaging/access-list/clear
  - messaging/access-list/get
---

## Abstract

The **Messaging — Remove from Access List** Trust Task removes one or more DIDs from a served account's *access list*. A mediator account carries a per-account access list — a set of other DIDs (VIDs) — which, combined with the account's [`MediatorAcl.accessListMode`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) (`explicitAllow` = an allowlist, `explicitDeny` = a denylist), governs who may send to that account. The *administrator* names the account in `payload.did` and supplies the DIDs to remove in `payload.entries`. The mediator applies its own authorization policy — an `admin`/`rootAdmin` requester may modify any account's access list; a `standard` account may modify only its own list, and only where it is permitted to self-manage it (`selfManageList`).

The remove is **idempotent at the set level**: an entry not present is silently ignored, and the response reports the entries actually removed (the set intersection) alongside the resulting access-list size.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/access-list/remove/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.entries` with at least one DID to remove.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/access-list/remove:unknownAccount`.
3. Where the requester is the account's own controller and the account is not permitted to self-manage its access list, respond with `messaging/access-list/remove:selfChangeDenied`, or with the framework's `permissionDenied` where the requester has no standing at all.
4. Otherwise remove each supplied entry that is present, ignore those that are not, and return the entries actually removed and the resulting access-list size.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/access-list/remove/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/access-list/remove/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "entries": [
      "did:web:bob.example"
    ]
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

This example removes Bob from Alice's access list.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/access-list/remove/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, removed, accessListCount }`, where `removed` reports the entries actually removed (those that were present) and `accessListCount` is the resulting size of the access list.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/access-list/remove/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "removed": [
      "did:web:bob.example"
    ],
    "accessListCount": 6
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A remove-from-access-list document is an administrative mutation: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the change is permitted.

The access list combined with `accessListMode` directly governs delivery: removing an entry under `explicitAllow` revokes a sender's access, while removing under `explicitDeny` restores it. An administrator **SHOULD** confirm the account's `accessListMode` before removing, since the same change has opposite effect under each mode.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
