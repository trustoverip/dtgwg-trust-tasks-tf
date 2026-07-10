---
slug: messaging/access-list/get
version: "0.1"
title: Messaging — Check Access List Membership
summary: A requester checks whether specific DIDs are present in a served account's access list — the per-account set of other DIDs that, combined with the account's accessListMode, governs who may send to that account.
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
  requirement: RECOMMENDED
  rationale: This is a read-only membership check that mutates nothing; a proof is RECOMMENDED so the requester may be authenticated and the response bound to a specific request where the ecosystem relies on it after the transport has closed, but it is not required for a query.
sideEffects:
  level: none
  rationale: "Read-only membership check against an account's access list."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: messaging/access-list/get:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
related:
  - messaging/access-list/list
  - messaging/access-list/add
  - messaging/access-list/remove
---

## Abstract

The **Messaging — Check Access List Membership** Trust Task tests whether one or more specific DIDs are present in a served account's *access list*. A mediator account carries a per-account access list — a set of other DIDs (VIDs) — which, combined with the account's [`MediatorAcl.accessListMode`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) (`explicitAllow` = an allowlist, `explicitDeny` = a denylist), governs who may send to that account. The *requester* names the account in `payload.did` and supplies the DIDs to test in `payload.entries`; the mediator partitions them into those present and those absent. This is a **read-only** query and changes nothing.

Use this task to check a known set of candidates without paging the whole list ([`messaging/access-list/list`](../../list/0.1/spec.md)); the response reports membership only, and does not interpret it against `accessListMode`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/access-list/get/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID and `payload.entries` with at least one DID to test.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) where the ecosystem relies on the response (**RECOMMENDED**).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and, where a `proof` is present, verify it.
2. Where the target DID has no account, respond with `messaging/access-list/get:unknownAccount`.
3. Apply its own read-authorization policy, responding with the framework's `permissionDenied` where the requester has no standing to inspect the account's access list.
4. Otherwise partition the supplied entries into those present in the access list and those absent, and return both sets.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/access-list/get/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/access-list/get/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "entries": [
      "did:web:bob.example",
      "did:web:dave.example"
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

This example checks whether Bob and Dave are on Alice's access list.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/access-list/get/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, present, absent }`, partitioning the supplied entries into those present in the access list and those absent.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/access-list/get/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "present": [
      "did:web:bob.example"
    ],
    "absent": [
      "did:web:dave.example"
    ]
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A membership check is read-only and mutates nothing; `proof` is **RECOMMENDED** rather than required, and serves to authenticate the requester and bind the response where the ecosystem relies on it after the transport has closed.

An access list reveals who an account communicates with, and a membership check leaks that relationship one probe at a time. A mediator **MUST** enforce its own read-authorization policy independent of the document, and **SHOULD** disclose membership only to the account's controller or an administrator. The response reports membership only; it does not interpret presence against `accessListMode`, so a requester **MUST NOT** infer send permission from a `present`/`absent` result without also knowing the account's mode.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
