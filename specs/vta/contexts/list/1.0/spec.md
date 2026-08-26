---
slug: vta/contexts/list
version: "1.0"
title: VTA Contexts — List
summary: A caller enumerates the VTA contexts it can reach — the separation boundary that keys, DIDs, vault entries and policy all belong to.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - context
  - list
  - scope
  - separation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: A read with no durable state change; the transport's authenticated sender is what the VTA filters on. A proof MAY be included where the response is retained for audit.
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/contexts/get
  - vta/contexts/create
  - vta/contexts/update
  - vta/contexts/delete
---

## Abstract

A **context** is the VTA's unit of separation. Keys, DIDs, vault entries,
policies and memory each belong to exactly one, an ACL entry's scopes are
context ids, and contexts nest — so authority granted at a parent reaches its
descendants. **VTA Contexts — List** returns the contexts the caller can reach.

The result is filtered rather than refused: a caller with access to one context
sees one, and a caller with access to none sees an empty array. That makes the
response say what the caller may act on without disclosing the shape of the
rest of the VTA.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

This specification documents a task already deployed at this version. It is
being written down after the fact, and where the wire and this document differ
the difference is a defect in one of them, to be resolved rather than assumed.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** return only contexts the
authenticated caller may access, and **MUST NOT** signal the existence of
contexts it filtered out — neither by a count, a gap in ordering, nor an error.

A conforming **consumer** **MUST** return `basePath` as it derives it from the
parent chain. A producer that supplies `basePath` on any task in this family is
supplying derived state, and the consumer **MUST** ignore it rather than store
it.

## Authorization

This specification targets framework `0.2`, which does not oblige it to carry
this section; it is included because the filtering rule above is only
comprehensible alongside the authority it implements.

Authority to see a context is **context access** — the same ACL scope
resolution that gates keys and vault entries within it, extended down the
parent chain. There is no separate list capability: a caller that may act in a
context may see it, and no other caller learns it exists.

Verifying the producer's VID or `proof` establishes *who is asking*, never
*what they may see* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The scope
resolution is the authorization, and it happens after identity is settled.

## Request

An empty payload. There are no filters, because the answer is already the
caller's own reachable set.

```json
{
  "id": "6f1a9d2c-4c1e-4f0e-9c1a-2b7d5e0f3a11",
  "type": "https://trusttasks.org/spec/vta/contexts/list/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:00:00Z",
  "payload": {}
}
```

## Response

The VTA answers with every context the caller may reach.

```json
{
  "id": "d3b2c1a0-8e7f-4a6b-9c5d-1e2f3a4b5c6d",
  "type": "https://trusttasks.org/spec/vta/contexts/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:00:01Z",
  "threadId": "6f1a9d2c-4c1e-4f0e-9c1a-2b7d5e0f3a11",
  "payload": {
    "contexts": [
      {
        "id": "personal",
        "name": "Personal",
        "did": "did:webvh:QmScid:example.com",
        "basePath": "personal",
        "createdAt": "2026-01-04T10:00:00Z",
        "updatedAt": "2026-07-19T14:22:00Z"
      },
      {
        "id": "personal/banking",
        "name": "Banking",
        "parent": "personal",
        "basePath": "personal/banking",
        "createdAt": "2026-03-11T08:30:00Z",
        "updatedAt": "2026-03-11T08:30:00Z"
      }
    ]
  }
}
```

A caller who may reach nothing receives `{"contexts": []}` with no error. That
is the whole of the answer: an empty list here is not evidence that the VTA
holds no contexts.

## Security & Privacy

The membership of this list is itself information. A VTA that answered
`forbidden` for a context the caller cannot see — rather than omitting it —
would confirm that context's existence to anyone who guessed its id, which is
why the filtering rule above is normative rather than an implementation choice.

`ContextPolicy` is deliberately **not** part of the record this task returns.
Policy describes what a context restricts, and a caller who may see that a
context exists does not thereby need to know how it is fenced; a caller who
needs the policy asks [`vta/contexts/get`](../../get/1.0/spec.md) for that one
context.

**Free text.** Each returned record's `name` is free text, bounded at 256
characters — a display name, not prose. It was authored by whichever operator
created or last updated that context, is read by whoever reads this listing, and
is **retained** by the VTA for the life of the context. It is operator-facing
only and carries no authorization meaning; two contexts may share a name, so a
caller MUST match on the context identifier rather than on `name`.

