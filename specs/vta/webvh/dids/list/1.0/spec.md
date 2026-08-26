---
slug: vta/webvh/dids/list
version: "1.0"
title: "VTA WebVH DIDs — List"
summary: "Enumerate the did:webvh DIDs a VTA holds, optionally narrowed to a context or hosting server."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - list
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
  rationale: "A read with no durable state change."
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/webvh/dids/get
  - vta/webvh/dids/create
  - vta/webvh/servers/list
---

## Abstract

**VTA WebVH DIDs — List** enumerates the `did:webvh` DIDs a VTA holds, filtered
to what the caller can reach and optionally narrowed to one context or hosting
server.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** return only DIDs whose context the
caller may access, and **MUST NOT** signal the existence of those it filtered
out.

An empty array is a **successful** answer. A caller needing to know whether a
specific DID exists asks [`vta/webvh/dids/get`](../../get/1.0/spec.md), which
answers `notFound`.

## Authorization

Authority is **context access**, resolved per DID. There is no separate
enumeration capability.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/list/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": { "contextId": "personal" }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "dids": [
      {
        "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
        "serverId": "prod",
        "mnemonic": "alice",
        "scid": "QmScidAbCdEfGh",
        "contextId": "personal",
        "portable": true,
        "logEntryCount": 4,
        "preRotationCount": 0,
        "nextFragmentId": 1,
        "createdAt": "2026-08-19T11:00:01Z",
        "updatedAt": "2026-08-19T12:30:00Z"
      }
    ]
  }
}
```

## Security & Privacy

The set of DIDs a VTA holds says which identities one operator controls, which
is exactly the correlation a per-context separation exists to prevent. Filtering
rather than refusing keeps that inference inside the boundary the ACL already
draws.
