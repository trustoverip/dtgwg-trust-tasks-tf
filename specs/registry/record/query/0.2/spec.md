---
slug: registry/record/query
version: "0.2"
title: Registry — Query Records
summary: An administrator fetches one trust record by its full four-part key, or enumerates matching records with cursor pagination — one task for both halves of the superseded read/list pair.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - trust-registry
  - trqp
  - record
  - query
  - pagination
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
  rationale: Querying records is a non-mutating administrative operation whose integrity is normally guaranteed by the transport; a proof is RECOMMENDED, not REQUIRED.
sideEffects:
  level: none
  rationale: "Read-only fetch or enumeration of trust records."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: registry/record/query:notFound
    meaning: All four key parts were supplied (exact fetch) and no record exists for that entity+authority+action+resource key. A partially keyed enumeration never raises this — it returns an empty page instead.
    retryable: false
related:
  - registry/record/put
  - registry/record/delete
  - registry/recognition
  - registry/authorization
---

## Abstract

The **Registry — Query Records** Trust Task returns stored [`TrustRecord`](../../../_shared/0.1/registry.schema.json)s from a trust registry. The four TRQP key fields (`entity_id`, `authority_id`, `action`, `resource`) are each OPTIONAL:

* **All four supplied** — an exact fetch of the single record at that key. A miss is an error (`registry/record/query:notFound`), preserving the superseded [`registry/record/read`](../../read/0.1/spec.md) semantics that admin tooling relies on to distinguish "absent" from "empty".
* **Fewer than four supplied** — a filtered enumeration of every record matching the supplied parts (none supplied: all records), paginated by `cursor`/`limit`. This fixes the pagination gap the superseded [`registry/record/list`](../../list/0.1/spec.md) conceded in its own abstract; an empty page is a normal response, not an error.

Unlike the TRQP [`recognition`](../../../recognition/0.1/spec.md) and [`authorization`](../../../authorization/0.1/spec.md) query responses — which each expose only their own facet — query returns complete records (both `recognized` and `authorized` where present) for administrative inspection.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

- Response **`nextCursor`** narrows from `string | null` to **`string`**. On the last page of an enumeration, and on a fully keyed fetch, the registry omits it; it is never `null`.

This applies the registry-wide pagination convention, under which the absence of `nextCursor` is the one way to say "last page". A registry that sent `"nextCursor": null` under `0.1` omits the member instead. The request is unchanged, and nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

Each response entry follows the shared [`TrustRecord`](../../../_shared/0.1/registry.schema.json) definition. The four key fields are verbatim TRQP (hence snake_case, as throughout the `registry/*` family); the pagination fields follow the framework 0.2 camelCase convention.

`cursor` is an opaque continuation token from a previous page's `nextCursor`; producers MUST NOT construct or interpret it. `limit` is the requested page size; the registry clamps to 1..=200 (default 50).

## Request

The *registry administrator* (document `issuer`) sends the (possibly partial) key and pagination parameters to the *trust registry* (document `recipient`).

### Example request — exact fetch

```json
{
  "entity_id": "did:example:issuer-42",
  "authority_id": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential"
}
```

### Example request — paginated enumeration

```json
{
  "authority_id": "did:web:ecosystem.example",
  "limit": 50
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) carrying the `records` array and, on an enumeration with more matches remaining, a `nextCursor` the producer echoes back to continue. On the last page `nextCursor` is absent, never `null`. Enumeration order MUST be stable across pages of one traversal; records put or deleted mid-traversal MAY or MAY NOT be reflected.

On a fully keyed fetch, `records` carries exactly one entry and `nextCursor` is absent; a miss returns `trust-task-error` with `registry/record/query:notFound` (not an empty `records`).

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

### Data carried

The request carries up to four key parts (entity, authority, action, resource)
and paging members. The response carries complete
[`TrustRecord`](../../../_shared/0.1/registry.schema.json)s — both `recognized`
and `authorized` where present, and the opaque governance `context`. Querying exposes the registry's governance dataset — up to all of it on an unfiltered enumeration; restrict to authorized administrators and carry over an authenticated, confidential transport.

### Correlation

Each record joins an entity identifier to the authorities that recognize or
authorize it and to the actions and resources concerned; an enumeration
filtered on `entity_id` returns that entity's whole standing in the registry.
`threadId` correlates request and response, and `nextCursor`/`cursor` link the
pages of one traversal. Cursors SHOULD NOT encode sensitive record content in recoverable form; an index position or a keyed digest suffices.

### Retention

The task is a read-only fetch or enumeration and persists nothing at the
registry. The specification imposes no retention rule on the administrator's
copy of the response.

### Consent/purpose

The purpose is administrative inspection of the registry's stored records.
Resolvers asking recognition or authorization questions use the TRQP
[`recognition`](../../../recognition/0.1/spec.md) and
[`authorization`](../../../authorization/0.1/spec.md) queries, which expose
only their own facet. The specification states no further purpose limitation.
