---
slug: registry/record/put
version: "0.1"
title: Registry — Put Record
summary: An administrator creates or replaces, in a verifiable form, the recognition or authorization record at a trust registry key — one task for both halves of the superseded create/update pair.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - put
  - upsert
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
  rationale: Putting a trust record is an evidentiary, state-changing administrative act that may be audited or replayed after the original transport has closed; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Creates or replaces a trust record; deletable and re-puttable."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: registry/record/put:alreadyExists
    meaning: "`expectedExisting: false` was asserted but a record already exists for the given entity+authority+action+resource key."
    retryable: false
  - code: registry/record/put:notFound
    meaning: "`expectedExisting: true` was asserted but no record exists for the given entity+authority+action+resource key."
    retryable: false
related:
  - registry/record/query
  - registry/record/delete
  - registry/recognition
  - registry/authorization
  - vault/upsert
---

## Abstract

The **Registry — Put Record** Trust Task creates or replaces a trust record — a recognition or authorization assertion — in a trust registry. The *registry administrator* declares the [`TrustRecord`](../../../_shared/0.1/registry.schema.json) to store; the *trust registry* applies its own policy and, if accepted, stores it at the record's four-part key (`entity_id`, `authority_id`, `action`, `resource`), overwriting any record already there.

This task supersedes the [`registry/record/create`](../../create/0.1/spec.md) / [`registry/record/update`](../../update/0.1/spec.md) pair, which carried the identical single-`record` payload and differed only in their already-exists / not-found error codes. Following the [`vault/upsert`](../../../../vault/upsert/0.2/spec.md) precedent, those strict semantics remain available through the optional `expectedExisting` assertion — so nothing expressible before is lost, and the common case is one round trip with no prior existence check.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

`record` follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition: the four identifiers, a `record_type` (`recognition` or `authorization`), the corresponding `recognized`/`authorized` boolean, and an opaque governance `context`. Its field names are verbatim from the ToIP TRQP v2.0 schemas (hence snake_case, as throughout the `registry/*` family).

`expectedExisting` is an OPTIONAL existence assertion:

* **absent** — pure create-or-update: the registry stores the record whether or not the key already exists.
* **`true`** — strict update: the key MUST already exist; otherwise the registry MUST reject with `registry/record/put:notFound`.
* **`false`** — strict create: the key MUST NOT already exist; otherwise the registry MUST reject with `registry/record/put:alreadyExists`.

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

The *trust registry* (now the `issuer` of the response) replies with the `#response` document reachable via `$anchor: "response"` in `payload.schema.json`, carrying `ok: true` and `created` — `true` when the put created a new record, `false` when it replaced an existing one. A violated `expectedExisting` assertion uses `trust-task-error` with the corresponding error code, not a `#response` document.

### Example response

```json
{ "ok": true, "created": true }
```

## Security & Privacy

Only administrators authorized by the registry's own ACL may put records; the registry MUST verify the `proof` and the authenticated sender before applying the change. Records are evidentiary; the registry SHOULD retain an audit trail of put/delete operations that records whether each put created or replaced.

Because a bare put silently replaces whatever is at the key, administrators racing on the same key can overwrite each other; tooling that requires lost-update protection SHOULD assert `expectedExisting` (and compare the read-back record via [`registry/record/query`](../../query/0.1/spec.md)) rather than issuing blind puts.
