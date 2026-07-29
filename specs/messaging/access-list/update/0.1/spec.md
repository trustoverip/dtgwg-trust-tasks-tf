---
slug: messaging/access-list/update
version: "0.1"
title: Messaging — Update Access List
summary: An administrator modifies a served account's access list in one task — clear it, add entries, and remove entries — replacing the separate add, remove, and clear tasks.
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
  rationale: Modifying an account's access list is an administrative mutation whose record may be replayed by an auditor or relied on after the original transport has closed; transport-independent integrity and non-repudiation of the change are required.
sideEffects:
  level: mutating
  rationale: "Adds and/or removes access-list entries, optionally clearing the list first; individual entries are re-addable."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/access-list/update:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
  - code: messaging/access-list/update:listFull
    meaning: Applying the additions would exceed the account's configured maximum access-list size; no change was made.
    retryable: false
  - code: messaging/access-list/update:selfChangeDenied
    meaning: The requester is the account's own controller and the account is not permitted to self-manage its access list (selfManageList is not set).
    retryable: false
related:
  - messaging/access-list/list
  - messaging/account/update
  - messaging/account/get
---

## Abstract

The **Messaging — Update Access List** Trust Task modifies a served account's *access list* in a single request. A mediator account carries a per-account access list — a set of other DIDs (VIDs) — which, combined with the account's [`MediatorAcl.accessListMode`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) (`explicitAllow` = an allowlist, `explicitDeny` = a denylist), governs who may send to that account. The *administrator* names the account in `payload.did` and supplies any combination of `clear`, `add`, and `remove`. The mediator applies them in that fixed order — **clear first, then add, then remove** — so `{ "clear": true, "add": [...] }` replaces the list wholesale, and a plain `add` or `remove` behaves exactly like the single-verb tasks this one supersedes (`messaging/access-list/add`, `remove`, and `clear`).

The update is **idempotent at the set level**: an added entry already present is not duplicated, a removed entry not present is ignored, and the response reports the entries actually added and removed alongside the resulting list size. The mediator applies its own authorization policy — an `admin`/`rootAdmin` requester may modify any account's access list; a `standard` account may modify only its own list, and only where it is permitted to self-manage it (`selfManageList`).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/access-list/update/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and at least one of `payload.clear`, `payload.add`, `payload.remove`. A producer **SHOULD NOT** name the same DID in both `add` and `remove`; where it does, the fixed apply order means the entry ends up removed.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the target DID has no account, respond with `messaging/access-list/update:unknownAccount`.
3. Where the requester is the account's own controller and the account is not permitted to self-manage its access list, respond with `messaging/access-list/update:selfChangeDenied`, or with the framework's `permissionDenied` where the requester has no standing at all.
4. Where applying the additions would exceed the account's configured maximum access-list size, change nothing and respond with `messaging/access-list/update:listFull`.
5. Otherwise apply the present members in the fixed order **`clear`, `add`, `remove`** as one atomic change, and return the entries actually added, the entries actually removed by `remove`, and the resulting access-list size.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/access-list/update/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/access-list/update/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "add": [
      "did:web:bob.example",
      "did:web:carol.example"
    ],
    "remove": [
      "did:web:mallory.example"
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

This example adds Bob and Carol to Alice's access list and removes Mallory, in one signed change.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/access-list/update/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, added, removed, accessListCount }`: `added` reports the entries actually added, `removed` the entries actually removed by `remove` (entries dropped by `clear` are not enumerated), and `accessListCount` the resulting size of the list.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/access-list/update/0.1#response",
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
    "removed": [
      "did:web:mallory.example"
    ],
    "accessListCount": 7
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

An access-list update is an administrative mutation: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the change is permitted.

The access list combined with `accessListMode` directly governs delivery, and the same entry has the **opposite effect** under each mode: adding under `explicitAllow` grants a sender, adding under `explicitDeny` blocks one. An administrator **SHOULD** confirm the account's mode (via [`messaging/account/get`](../../../account/get/0.1/spec.md)) before updating. `clear` deserves particular care — on an `explicitAllow` list it silences the account entirely, and on an `explicitDeny` list it opens it to every sender; the entries it drops are not individually reported, so a mediator **SHOULD** audit a `clear` distinctly.

An access list reveals who an account communicates with; a mediator **SHOULD** treat list contents as sensitive and disclose them only to the account's controller or an administrator.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
