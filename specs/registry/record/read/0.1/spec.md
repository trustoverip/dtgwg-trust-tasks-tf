---
slug: registry/record/read
version: "0.1"
title: Registry — Read Record
summary: An administrator reads the full stored trust record for a given key, including both the recognition and authorization facets.
status: retired
supersededBy: registry/record/query
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - read
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Registry administrator
    requirement: REQUIRED
    member: issuer
  - role: Trust registry
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Reading a record is a non-mutating administrative operation whose integrity is normally guaranteed by the transport; a proof is RECOMMENDED, not REQUIRED.
sideEffects:
  level: none
  rationale: "Read-only read of a single trust record."
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - registry/record/list
  - registry/recognition
  - registry/authorization
---

## Abstract

The **Registry — Read Record** Trust Task returns the full stored [`TrustRecord`](../../../_shared/0.1/registry.schema.json) for a key (`entity_id`, `authority_id`, `action`, `resource`). Unlike the TRQP [`recognition`](../../../recognition/0.1/spec.md) and [`authorization`](../../../authorization/0.1/spec.md) query responses — which each expose only their own facet — the read returns the complete record (both `recognized` and `authorized` where present) for administrative inspection.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels) and is preserved for historical reference; its schema and prose are frozen. It is superseded by [`registry/record/query`](../../query/0.1/spec.md), which preserves the exact-fetch semantics (notFound on a fully keyed miss). Producers SHOULD NOT emit new documents against this specification.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The response follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition.

## Request

The *registry administrator* (document `issuer`) sends the record key to the *trust registry* (document `recipient`).

### Example request

```json
{
  "entity_id": "did:example:issuer-42",
  "authority_id": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential"
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying the full `TrustRecord`. A missing key returns `trust-task-error` (not a `#response`).

### Example response

```json
{
  "entity_id": "did:example:issuer-42",
  "authority_id": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential",
  "recognized": true,
  "authorized": true,
  "record_type": "authorization",
  "context": {}
}
```

## Security & Privacy

Reading records exposes the registry's full governance data; restrict to authorized administrators and carry over an authenticated, confidential transport.
