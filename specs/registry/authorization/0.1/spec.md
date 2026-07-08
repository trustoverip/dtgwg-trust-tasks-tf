---
slug: registry/authorization
version: "0.1"
title: Registry — Authorization Query
summary: A relying party asks a trust registry whether an entity is authorized by an authority for a given action and resource, per the ToIP Trust Registry Query Protocol (TRQP) v2.0.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - trust-registry
  - trqp
  - authorization
  - query
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Relying party
    requirement: REQUIRED
    member: issuer
  - role: Trust registry
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: An authorization query is a read whose integrity is normally guaranteed by the transport (authenticated DIDComm/TSP or HTTPS). A proof is RECOMMENDED, not REQUIRED, so the query remains usable on bindings without an in-band verifier.
related:
  - registry/recognition
  - registry/record/read
---

## Abstract

The **Registry — Authorization Query** Trust Task carries a ToIP [TRQP v2.0](https://trustoverip.github.io/tswg-trust-registry-protocol/) authorization query: the *relying party* asks the *trust registry* whether an entity (`entity_id`) is authorized by an authority (`authority_id`) to perform an `action` on a `resource`. The registry answers with a `#response` document carrying the boolean `authorized` and the evaluation time.

The `action` and `resource` strings are opaque to the framework — each authority defines their vocabulary in its governance framework. The payload field names are verbatim from the TRQP authorization request/response schemas, so a single shape serves both the plain HTTP TRQP binding (`POST /authorization`) and this Trust Task binding.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The `context`, when present, follows the shared [`QueryContext`](../../_shared/0.1/registry.schema.json) definition: `time` requests evaluation as of an RFC3339 instant, and `locator` is an authority-defined hint for locating the records in question.

## Request

The *relying party* (document `issuer`) sends the query to the *trust registry* (document `recipient`). The request carries the four TRQP identifiers `entity_id`, `authority_id`, `action`, `resource` (all REQUIRED) and an optional `context`.

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

The *trust registry* (now the `issuer` of the response) replies with the `#response` document reachable via `$anchor: "response"` in `payload.schema.json`. It echoes the four identifiers and adds `authorized` (REQUIRED) and `time_evaluated` (REQUIRED), plus the optional `time_requested`, `message`, and `context`. Failures use `trust-task-error`, not a `#response` document.

### Example response

```json
{
  "entity_id": "did:example:issuer-42",
  "authority_id": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential",
  "authorized": true,
  "time_evaluated": "2026-07-09T00:00:00Z",
  "message": "did:example:issuer-42 authorized to issue+https://schema.example/PersonhoodCredential by did:web:ecosystem.example"
}
```

## Security & Privacy

The query and response reveal which entity/authority/action/resource a relying party is interested in; carry them over an authenticated, confidential transport. The registry SHOULD apply its own access-control policy to who may query. Because the response is a point-in-time read, consumers SHOULD NOT cache it beyond the authority's stated validity window.
