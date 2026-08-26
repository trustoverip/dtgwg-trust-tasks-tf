---
slug: vta/contexts/create
version: "1.0"
title: VTA Contexts — Create
summary: An administrator creates a VTA context — a new separation boundary for keys, DIDs, vault entries and policy, optionally nested under an existing one.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - context
  - create
  - scope
  - separation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Creating a context creates a scope that ACL grants will later name, so the VTA must attribute it to a specific administrator in the audit record — independently of the transport that carried it.
sideEffects:
  level: mutating
  rationale: "Creates a durable context. Re-creatable, but grants made against it are not."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/create:idConflict
    meaning: A context with this id already exists under the same parent.
    retryable: false
  - code: vta/contexts/create:parentNotFound
    meaning: The named parent context does not exist or is not reachable by the caller.
    retryable: false
related:
  - vta/contexts/list
  - vta/contexts/get
  - vta/contexts/update
  - vta/contexts/delete
---

## Abstract

**VTA Contexts — Create** adds a [context](../../list/1.0/spec.md): a new
separation boundary that keys, DIDs, vault entries, policies and memory can
belong to, and that ACL entries can name as a scope.

A context may nest under an existing one. Nesting is not presentational —
authority granted at a parent reaches its descendants — so creating a child is
a decision about who can already reach the thing being created.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse an id that already exists
under the same parent with `vta/contexts/create:idConflict`, and **MUST NOT**
treat the request as an update. A caller that meant to change an existing
context asks [`vta/contexts/update`](../../update/1.0/spec.md).

When `parent` is present, the resulting context's `id` is `<parent>/<id>` and
`basePath` is derived from the full parent chain. The consumer **MUST** compute
both; a producer does not supply them.

A conforming **consumer** **MUST** refuse a `parent` the caller cannot reach
with `vta/contexts/create:parentNotFound` — the same answer it gives for a
parent that does not exist, for the reason set out in
[`vta/contexts/get`](../../get/1.0/spec.md).

## Authorization

Authority is the **administrator role at the parent scope**: creating a
top-level context requires administrative authority over the VTA, and creating
a child requires it over the parent whose authority the child will inherit.

The required `proof` establishes *who authored the request*, so that the
resulting context can be attributed in the audit record. It is not the
authorization — a correctly signed request from a caller without the role is
refused, and the role check happens after the signature is settled
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Request

```json
{
  "id": "1a2b3c4d-5e6f-4708-8192-a3b4c5d6e7f8",
  "type": "https://trusttasks.org/spec/vta/contexts/create/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:20:00Z",
  "payload": {
    "id": "banking",
    "name": "Banking",
    "description": "Accounts and payment credentials",
    "parent": "personal"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T09:20:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

The realized record, at the top level. Note that the id the VTA returns is the
full path, not the leaf the producer sent.

```json
{
  "id": "2b3c4d5e-6f70-4819-92a3-b4c5d6e7f809",
  "type": "https://trusttasks.org/spec/vta/contexts/create/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T09:20:01Z",
  "threadId": "1a2b3c4d-5e6f-4708-8192-a3b4c5d6e7f8",
  "payload": {
    "id": "personal/banking",
    "name": "Banking",
    "description": "Accounts and payment credentials",
    "parent": "personal",
    "basePath": "personal/banking",
    "createdAt": "2026-08-19T09:20:01Z",
    "updatedAt": "2026-08-19T09:20:01Z"
  }
}
```

## Security & Privacy

A context created under a parent is reachable by everyone who holds authority
at that parent, immediately and without a further grant. That is the intended
semantics of nesting, and it is the thing most likely to surprise: a context
created for separation, nested under a widely-granted parent, separates
nothing.

Creating a context discloses nothing by itself. What it does is create a name
that later ACL grants will reference — and a grant naming a context that was
subsequently deleted and recreated is a grant over different contents, which is
why [`vta/contexts/delete`](../../delete/1.0/spec.md) is specified as it is.
