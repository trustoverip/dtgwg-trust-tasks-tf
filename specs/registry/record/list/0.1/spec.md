---
slug: registry/record/list
version: "0.1"
title: Registry — List Records
summary: An administrator lists all trust records held by the registry.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - list
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
  rationale: Listing records is a non-mutating administrative operation whose integrity is normally guaranteed by the transport; a proof is RECOMMENDED, not REQUIRED.
sideEffects:
  level: none
  rationale: "Read-only listing of trust records."
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - registry/record/read
---

## Abstract

The **Registry — List Records** Trust Task returns every [`TrustRecord`](../../../_shared/0.1/registry.schema.json) the registry holds. It is an administrative operation for reconciliation and audit; large registries SHOULD expect to add pagination in a future version.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

Each response entry follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition.

## Request

The *registry administrator* (document `issuer`) sends an empty request to the *trust registry* (document `recipient`).

### Example request

```json
{}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying the `records` array.

### Example response

```json
{
  "records": [
    {
      "entity_id": "did:example:issuer-42",
      "authority_id": "did:web:ecosystem.example",
      "action": "issue",
      "resource": "https://schema.example/PersonhoodCredential",
      "authorized": true,
      "record_type": "authorization",
      "context": {}
    }
  ]
}
```

## Security & Privacy

Listing exposes the registry's full governance dataset; restrict to authorized administrators and carry over an authenticated, confidential transport.
