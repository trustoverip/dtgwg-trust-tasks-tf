---
slug: registry/record/create
version: "0.1"
title: Registry — Create Record
summary: An administrator records, in a verifiable form, a new recognition or authorization assertion in a trust registry.
status: retired
supersededBy: registry/record/put
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - create
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
  requirement: REQUIRED
  rationale: Creating a trust record is an evidentiary, state-changing administrative act that may be audited or replayed after the original transport has closed; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Records a new trust assertion; deletable."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: registry/record/create:already_exists
    meaning: A record already exists for the given entity+authority+action+resource key.
    retryable: false
related:
  - registry/record/update
  - registry/record/delete
  - registry/recognition
  - registry/authorization
---

## Abstract

The **Registry — Create Record** Trust Task records the addition of a new trust record — a recognition or authorization assertion — to a trust registry. The *registry administrator* declares the [`TrustRecord`](../../../_shared/0.1/registry.schema.json) to store; the *trust registry* applies its own policy and, if accepted, holds the record for subsequent TRQP queries.

The record is keyed by its four identifiers (`entity_id`, `authority_id`, `action`, `resource`). Creating a key that already exists is an error (`registry/record/create:already_exists`); changing an existing record uses [`registry/record/update`](../../update/0.1/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels) and is preserved for historical reference; its schema and prose are frozen. It is superseded by [`registry/record/put`](../../put/0.1/spec.md), which covers strict-create via `expectedExisting: false`. Producers SHOULD NOT emit new documents against this specification.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

`record` follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition: the four identifiers, a `record_type` (`recognition` or `authorization`), the corresponding `recognized`/`authorized` boolean, and an opaque governance `context`.

## Request

The *registry administrator* (document `issuer`) sends the record to the *trust registry* (document `recipient`). A `proof` is REQUIRED.

### Example request

```json
{
  "record": {
    "entity_id": "did:example:issuer-42",
    "authority_id": "did:web:ecosystem.example",
    "action": "issue",
    "resource": "https://schema.example/PersonhoodCredential",
    "authorized": true,
    "record_type": "authorization",
    "context": {}
  }
}
```

## Response

The *trust registry* (now the `issuer` of the response) replies with the `#response` document reachable via `$anchor: "response"` in `payload.schema.json`, carrying `ok: true`. Failures — including a duplicate key — use `trust-task-error` with `registry/record/create:already_exists`, not a `#response` document.

### Example response

```json
{ "ok": true }
```

## Security & Privacy

Only administrators authorized by the registry's own ACL may create records; the registry MUST verify the `proof` and the authenticated sender before applying the change. Records are evidentiary; the registry SHOULD retain an audit trail of create/update/delete operations.
