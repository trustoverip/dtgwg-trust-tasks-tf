---
slug: registry/record/delete
version: "0.1"
title: Registry — Delete Record
summary: An administrator deletes, in a verifiable form, a recognition or authorization record from a trust registry.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - delete
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
  rationale: Deleting a trust record is an evidentiary, state-changing administrative act that may be audited or replayed after the original transport has closed; transport-independent integrity is required.
errorCodes:
  - code: registry/record/delete:not_found
    meaning: No record exists for the given entity+authority+action+resource key.
    retryable: false
related:
  - registry/record/create
  - registry/record/update
---

## Abstract

The **Registry — Delete Record** Trust Task removes the trust record identified by its four TRQP identifiers (`entity_id`, `authority_id`, `action`, `resource`) from a trust registry. Deleting a non-existent key is an error (`registry/record/delete:not_found`).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The request carries the four TRQP identifiers that form the record key.

## Request

The *registry administrator* (document `issuer`) sends the key to the *trust registry* (document `recipient`). A `proof` is REQUIRED.

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

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying `ok: true`. A missing key returns `trust-task-error` with `registry/record/delete:not_found`.

### Example response

```json
{ "ok": true }
```

## Security & Privacy

Only administrators authorized by the registry's own ACL may delete records; the registry MUST verify the `proof` and the authenticated sender. The registry SHOULD retain an audit trail of the deletion.
