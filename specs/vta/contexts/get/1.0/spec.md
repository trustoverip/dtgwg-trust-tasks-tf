---
slug: vta/contexts/get
version: "1.0"
title: VTA Contexts — Get
summary: A caller fetches one VTA context by id, and learns definitively whether it exists.
status: draft
targetFrameworkVersion: "0.5"
category: did-management
keywords:
  - vta
  - context
  - get
  - scope
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
  rationale: A read with no durable state change; the transport's authenticated sender is what the VTA resolves scope against.
sideEffects:
  level: none
  rationale: "Single-record read."
subjectPath: /id
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/get:notFound
    meaning: No context with this id is reachable by the caller. Deliberately does not distinguish "does not exist" from "exists but not yours".
    retryable: false
related:
  - vta/contexts/list
  - vta/contexts/create
  - vta/contexts/update
  - vta/contexts/delete
---

## Abstract

**VTA Contexts — Get** returns one [context](../../list/1.0/spec.md) by id.

It exists alongside `list` because the two reads fail differently, which is the
distinction [SPEC's read-one/read-many guidance](../../../../../CONTRIBUTING-SPECS.md)
exists to preserve: a `get` for an unknown id is a definite `notFound` the
caller can act on, while a `list` that omits the id is a *successful* response
indistinguishable from "exists, filtered out".

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** answer `vta/contexts/get:notFound`
for an id the caller cannot reach, **whether or not a context with that id
exists**. Distinguishing the two — by a different code, a different latency
class, or a message — discloses the existence of a context to a caller who was
refused it.

The response payload is the context record **at the top level**, not wrapped in
a member.

## Authorization

Authority is **context access**: the ACL scope resolution that gates the
context's contents, extended down the parent chain, exactly as in
[`vta/contexts/list`](../../list/1.0/spec.md). There is no separate read
capability for the record itself.

A `proof`, where present, establishes that the producer authored the request —
not that it may read the context. The scope resolution is the authorization.

## Request

```json
{
  "id": "b41e6f80-2c3d-4a5b-8e9f-0a1b2c3d4e5f",
  "type": "https://trusttasks.org/spec/vta/contexts/get/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": { "id": "personal/banking" }
}
```

## Response

```json
{
  "id": "c52f7091-3d4e-4b6c-9f0a-1b2c3d4e5f60",
  "type": "https://trusttasks.org/spec/vta/contexts/get/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "b41e6f80-2c3d-4a5b-8e9f-0a1b2c3d4e5f",
  "payload": {
    "id": "personal/banking",
    "name": "Banking",
    "parent": "personal",
    "basePath": "personal/banking",
    "createdAt": "2026-03-11T08:30:00Z",
    "updatedAt": "2026-03-11T08:30:00Z"
  }
}
```

An unreachable or unknown id is refused, not answered with an empty record:

```json
{
  "id": "d63a8102-4e5f-4c7d-a01b-2c3d4e5f6071",
  "type": "https://trusttasks.org/spec/trust-task-error/0.2",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "b41e6f80-2c3d-4a5b-8e9f-0a1b2c3d4e5f",
  "payload": {
    "code": "vta/contexts/get:notFound",
    "message": "no such context"
  }
}
```

## Security & Privacy

The `notFound`-for-both rule above is the whole of this task's privacy
posture, and it is worth being explicit about what it costs: a legitimate
administrator who mistypes an id gets the same answer as an outsider probing
for one. That is the intended trade — the alternative turns this task into an
existence oracle for context ids.

**Free text.** The returned record's `name` is free text, bounded at 256
characters — a display name, not prose. It was authored by whichever operator
created or last updated the context, is read by whoever reads this response, and
is **retained** by the VTA for the life of the context. It is operator-facing
only and carries no authorization meaning; two contexts may share a name, so a
caller MUST match on the context identifier rather than on `name`.

