---
slug: registry/record/update
version: "0.1"
title: Registry — Update Record
summary: An administrator updates, in a verifiable form, an existing recognition or authorization record in a trust registry.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - update
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
  rationale: Updating a trust record is an evidentiary, state-changing administrative act that may be audited or replayed after the original transport has closed; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Updates an existing trust record; recoverable by updating again."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: registry/record/update:not_found
    meaning: No record exists for the given entity+authority+action+resource key.
    retryable: false
related:
  - registry/record/create
  - registry/record/delete
---

## Abstract

The **Registry — Update Record** Trust Task replaces the stored [`TrustRecord`](../../../_shared/0.1/registry.schema.json) for an existing key (`entity_id`, `authority_id`, `action`, `resource`). The key MUST already exist; updating a non-existent key is an error (`registry/record/update:not_found`) — create it with [`registry/record/create`](../../create/0.1/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

`record` follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition.

## Request

The *registry administrator* (document `issuer`) sends the updated record to the *trust registry* (document `recipient`). A `proof` is REQUIRED.

### Example request

```json
{
  "record": {
    "entity_id": "did:example:issuer-42",
    "authority_id": "did:web:ecosystem.example",
    "action": "issue",
    "resource": "https://schema.example/PersonhoodCredential",
    "authorized": false,
    "record_type": "authorization",
    "context": {}
  }
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying `ok: true`. A missing key returns `trust-task-error` with `registry/record/update:not_found`.

### Example response

```json
{ "ok": true }
```

## Security & Privacy

Only administrators authorized by the registry's own ACL may update records; the registry MUST verify the `proof` and the authenticated sender. The registry SHOULD retain an audit trail of the change.
