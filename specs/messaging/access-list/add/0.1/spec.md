---
slug: messaging/access-list/add
version: "0.1"
title: Messaging — Add to Access List
summary: An administrator adds one or more DIDs to a served account's access list — the per-account set of other DIDs that, combined with the account's accessListMode, governs who may send to that account.
status: retired
supersededBy: messaging/access-list/update
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
  rationale: Adding entries to an account's access list is an administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Adds DIDs to an account's access list; removable."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/access-list/add:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/access-list/add:listFull
    meaning: The account's access list has reached its configured maximum size; no entries were added.
    retryable: false
  - code: messaging/access-list/add:selfChangeDenied
    meaning: The requester is the account's own controller and the account is not permitted to self-manage its access list (selfManageList is not set).
    retryable: false
related:
  - messaging/access-list/remove
  - messaging/access-list/get
  - messaging/acl/set
---

## Abstract

The **Messaging — Add to Access List** Trust Task adds one or more DIDs to a served account's *access list*. A mediator account carries a per-account access list — a set of other DIDs (VIDs) — which, combined with the account's [`MediatorAcl.accessListMode`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) (`explicitAllow` = an allowlist, `explicitDeny` = a denylist), governs who may send to that account. The *administrator* names the account in `payload.did` and supplies the DIDs to add in `payload.entries`. The mediator applies its own authorization policy — an `admin`/`rootAdmin` requester may modify any account's access list; a `standard` account may modify only its own list, and only where it is permitted to self-manage it (`selfManageList`).

The add is **idempotent at the set level**: an entry already present is not duplicated, and the response reports the entries actually added (the set difference) alongside the resulting access-list size.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by [`messaging/access-list/update`](../../update/0.1/spec.md) — send `{ did, add: entries }`; the idempotent set semantics, `listFull` / `selfChangeDenied` guards, and the `added` / `accessListCount` response members carry over unchanged.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/access-list/add/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.entries` with at least one DID to add.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/access-list/add:unknownAccount`.
3. Where the requester is the account's own controller and the account is not permitted to self-manage its access list, respond with `messaging/access-list/add:selfChangeDenied`, or with the framework's `permissionDenied` where the requester has no standing at all.
4. Where applying the additions would exceed the account's configured maximum access-list size, add nothing and respond with `messaging/access-list/add:listFull`.
5. Otherwise add each supplied entry not already present, and return the entries actually added and the resulting access-list size.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/access-list/add/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/access-list/add/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "entries": [
      "did:web:bob.example",
      "did:web:carol.example"
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

This example adds Bob and Carol to Alice's access list.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/access-list/add/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, added, accessListCount }`, where `added` reports the entries actually added (those not already present) and `accessListCount` is the resulting size of the access list.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/access-list/add/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "added": [
      "did:web:bob.example",
      "did:web:carol.example"
    ],
    "accessListCount": 7
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

An add-to-access-list document is an administrative mutation: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the change is permitted.

The access list combined with `accessListMode` directly governs delivery: adding an entry under `explicitAllow` grants a sender, while adding under `explicitDeny` denies one. An administrator **SHOULD** confirm the account's `accessListMode` (via [`messaging/acl/get`](../../../acl/get/0.1/spec.md) or [`messaging/account/get`](../../../account/get/0.1/spec.md)) before adding, since the same entry has opposite effect under each mode.

An access list reveals who an account communicates with; a mediator **SHOULD** treat list contents as sensitive and disclose them only to the account's controller or an administrator.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
