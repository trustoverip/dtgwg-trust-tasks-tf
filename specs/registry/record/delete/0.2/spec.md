---
slug: registry/record/delete
version: "0.2"
title: Registry — Delete Record
summary: An administrator deletes, in a verifiable form, a recognition or authorization record from a trust registry.
status: draft
targetFrameworkVersion: "0.6.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deleting a registry record removes what other parties resolve against. Replayed after the record was re-created it deletes the new one, and the delete carries no reference to the revision it was written about.
sideEffects:
  level: destructive
  rationale: "Deletes a trust record; relying parties querying it will no longer find the assertion."
consequences:
  - "Relying parties depending on this record lose the assertion immediately."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: registry/record/delete:notFound
    meaning: No record exists for the given entity+authority+action+resource key.
    retryable: false
related:
  - registry/record/put
  - registry/record/query
---

## Abstract

The **Registry — Delete Record** Trust Task removes the trust record identified by its four identifiers (`entityId`, `authorityId`, `action`, `resource`) from a trust registry. Deleting a non-existent key is an error (`registry/record/delete:notFound`).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

- Members are renamed to lowerCamelCase, as throughout the `registry/*` family: `entity_id` → `entityId`, `authority_id` → `authorityId`. Types, requiredness and meaning are unchanged.
- The response is unchanged.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The request carries the four identifiers that form the record key.

## Request

The *registry administrator* (document `issuer`) sends the key to the *trust registry* (document `recipient`). A `proof` is REQUIRED.

### Example request

```json
{
  "entityId": "did:example:issuer-42",
  "authorityId": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential"
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying `ok: true`. A missing key returns `trust-task-error` with `registry/record/delete:notFound`.

### Example response

```json
{ "ok": true }
```

## Security & Privacy

### Data carried

The request carries the four identifiers that form the record key. The response carries `ok` and an optional `message` of at most 1024 characters.

### Correlation

The key joins an entity identifier to the authority that recognized or authorized it. The document's `issuer` and its `proof` tie the deletion to the administrator who made it, and `threadId` correlates request and response.

### Retention

The registry SHOULD retain an audit trail of the deletion, naming the key and the administrator. The record itself is removed: a later query about any instant no longer finds it, so an administrator who wants a statement's end to stay answerable SHOULD instead put the record again with a `validUntil` ([`registry/record/put`](../../put/0.3/spec.md)).

### Consent/purpose

Only administrators authorized by the registry's own ACL may delete records; the registry MUST verify the `proof` and the authenticated sender before applying the change.
