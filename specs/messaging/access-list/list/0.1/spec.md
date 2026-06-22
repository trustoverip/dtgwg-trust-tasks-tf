---
slug: messaging/access-list/list
version: "0.1"
title: Messaging — List Access List
summary: A requester pages through a served account's full access list — the per-account set of other DIDs that, combined with the account's accessListMode, governs who may send to that account.
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
  rationale: This is a read-only enumeration that mutates nothing; a proof is RECOMMENDED so the requester may be authenticated and the response bound to a specific request where the ecosystem relies on it after the transport has closed, but it is not required for a query.
errorCodes:
  - code: messaging/access-list/list:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
related:
  - messaging/access-list/get
  - messaging/access-list/add
  - messaging/access-list/clear
---

## Abstract

The **Messaging — List Access List** Trust Task enumerates a served account's *access list* in pages. A mediator account carries a per-account access list — a set of other DIDs (VIDs) — which, combined with the account's [`MediatorAcl.accessListMode`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) (`explicitAllow` = an allowlist, `explicitDeny` = a denylist), governs who may send to that account. The *requester* names the account in `payload.did` and may bound the page with an optional `limit`; the mediator returns a page of entries and, where more remain, an opaque `nextCursor` to fetch the next page. This is a **read-only** query and changes nothing.

A first request omits `cursor`; each subsequent request echoes the `nextCursor` returned by the previous page. The `cursor` is **opaque** to the requester — its structure is the mediator's concern and a requester **MUST NOT** construct, parse, or modify it. Enumeration is complete when a response omits `nextCursor`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/access-list/list/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID; omit `cursor` on the first request and echo the previous page's `nextCursor` thereafter.
3. Treat `cursor` as opaque, and include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) where the ecosystem relies on the response (**RECOMMENDED**).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and, where a `proof` is present, verify it.
2. Where the target DID has no account, respond with `messaging/access-list/list:unknownAccount`.
3. Apply its own read-authorization policy, responding with the framework's `permissionDenied` where the requester has no standing to inspect the account's access list.
4. Otherwise return a page of entries bounded by `limit` (the mediator MAY apply a smaller server-side bound), include a `nextCursor` only where further entries remain, and report `accessListCount` as the total size of the list.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/access-list/list/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/access-list/list/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "limit": 2
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

This example requests the first page of Alice's access list, at most two entries.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/access-list/list/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ did, entries, accessListCount }` plus an optional `nextCursor`; `entries` is the current page and `accessListCount` is the total size of the list. `nextCursor` is present only where further entries remain.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/access-list/list/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "did": "did:web:alice.example",
    "entries": [
      "did:web:bob.example",
      "did:web:carol.example"
    ],
    "nextCursor": "eyJvIjoyfQ==",
    "accessListCount": 6
  }
}
```

A request echoing `nextCursor` returns the next page; the final page omits `nextCursor`. Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

An enumeration is read-only and mutates nothing; `proof` is **RECOMMENDED** rather than required, and serves to authenticate the requester and bind the response where the ecosystem relies on it after the transport has closed.

An access list reveals who an account communicates with; enumerating it discloses the full set at once. A mediator **MUST** enforce its own read-authorization policy independent of the document, and **SHOULD** disclose list contents only to the account's controller or an administrator.

The `cursor` is opaque and **SHOULD** be unguessable and bound to the requesting principal so it cannot be replayed by another party to page an account's list. A requester **MUST NOT** treat a page as a consistent snapshot — entries MAY be added or removed between pages — and **MUST NOT** interpret membership against `accessListMode` without also knowing the account's mode.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.
