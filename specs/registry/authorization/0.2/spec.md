---
slug: registry/authorization
version: "0.2"
title: Registry — Authorization Query
summary: A relying party asks a trust registry whether an entity is authorized by an authority for a given action and resource, per the ToIP Trust Registry Query Protocol (TRQP) v2.0.
status: draft
targetFrameworkVersion: "0.6.0"
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
sideEffects:
  level: none
  rationale: "Read-only TRQP authorization query."
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - registry/recognition
  - registry/record/query
---

## Abstract

The **Registry — Authorization Query** Trust Task carries a ToIP [TRQP v2.0](https://trustoverip.github.io/tswg-trust-registry-protocol/) authorization query: the *relying party* asks the *trust registry* whether an entity (`entityId`) is authorized by an authority (`authorityId`) to perform an `action` on a `resource`. The registry answers with a `#response` document carrying the boolean `authorized` and the evaluation time.

The `action` and `resource` strings are opaque to the framework — each authority defines their vocabulary in its governance framework. The query carries the same information as the TRQP authorization request and response; its members are lowerCamelCase, as throughout the `registry/*` family, and a registry that also serves the plain HTTP TRQP binding (`POST /authorization`) maps them to TRQP's snake_case names there (see [TRQP field names](#trqp-field-names)).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

- Members are renamed to lowerCamelCase: `entity_id` → `entityId`, `authority_id` → `authorityId`, `time_evaluated` → `evaluatedAt` and `time_requested` → `asOf`, following the registry's `<event>At` convention for instants and its `asOf` spelling for the instant a read is taken at. Types, requiredness and meaning are unchanged.
- The answer is evaluated against the matching record's validity window (`validFrom`/`validUntil` on the shared [`TrustRecord`](../../_shared/0.2/registry.schema.json) 0.2), as of `context.time` when supplied. 0.1 left the meaning of `context.time` to the registry.
- The query context, and the record shape, move to the shared schema [0.2](../../_shared/0.2/registry.schema.json).

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The `context`, when present, follows the shared [`QueryContext`](../../_shared/0.2/registry.schema.json) definition: `time` requests evaluation as of an RFC3339 instant, and `locator` is an authority-defined hint for locating the records in question.

The **evaluation instant** is `context.time` when it is present and not blank, and otherwise the registry's current time.

A record's **validity window** is the half-open interval from its `validFrom` (inclusive) to its `validUntil` (exclusive), as defined by the shared [`TrustRecord`](../../_shared/0.2/registry.schema.json). An absent bound leaves that side open, so a record with neither bound holds at every instant.

## Evaluation

The registry looks up the record at the four-part key (`entityId`, `authorityId`, `action`, `resource`).

- If the record's validity window contains the evaluation instant, `authorized` is the record's own `authorized` value.
- If the window does not contain it, `authorized` MUST be `false`. The registry SHOULD say why in `message`, for example that the statement is not yet, or no longer, valid.
- If no record exists at the key, `authorized` is `false`.

When the evaluation instant is the registry's own current time (no `context.time` was supplied), the registry MAY treat a record as having started up to 60 seconds before its `validFrom`, to absorb clock skew between the party that wrote the record and the registry. It MUST NOT extend `validUntil` in the same way: a statement that has ended is not honoured late. When `context.time` is supplied, the registry MUST evaluate at exactly that instant.

A registry keeps the records it currently holds, not their history. An evaluation instant in the past is judged against the current record's validity window; an earlier revision that a later `registry/record/put` replaced, or a record since deleted, is not consulted.

## Request

The *relying party* (document `issuer`) sends the query to the *trust registry* (document `recipient`). The request carries the four identifiers `entityId`, `authorityId`, `action`, `resource` (all REQUIRED) and an optional `context`.

### Example request

```json
{
  "entityId": "did:example:issuer-42",
  "authorityId": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential",
  "context": { "time": "2026-07-09T00:00:00Z" }
}
```

## Response

The *trust registry* (now the `issuer` of the response) replies with the `#response` document reachable via `$anchor: "response"` in `payload.schema.json`. It echoes the four identifiers and adds `authorized` (REQUIRED) and `evaluatedAt` (REQUIRED), plus the optional `asOf`, `message`, and `context`. `evaluatedAt` is the registry's clock when it produced the answer; `asOf` is the instant the answer holds for, echoing `context.time` when one was supplied. Failures use `trust-task-error`, not a `#response` document.

### Example response

```json
{
  "entityId": "did:example:issuer-42",
  "authorityId": "did:web:ecosystem.example",
  "action": "issue",
  "resource": "https://schema.example/PersonhoodCredential",
  "authorized": true,
  "asOf": "2026-07-09T00:00:00Z",
  "evaluatedAt": "2026-07-10T08:15:02Z",
  "message": "did:example:issuer-42 authorized to issue+https://schema.example/PersonhoodCredential by did:web:ecosystem.example"
}
```

## TRQP field names

A registry that serves both this task and the plain HTTP TRQP binding answers the same question on both. The members map one to one:

| This task | TRQP v2.0 |
|---|---|
| `entityId` | `entity_id` |
| `authorityId` | `authority_id` |
| `action`, `resource`, `context`, `authorized`, `message` | unchanged |
| `evaluatedAt` | `time_evaluated` |
| `asOf` | `time_requested` |

## Security & Privacy

### Data carried

The request carries the four identifiers (entity, authority, action, resource) and an optional `context` of string members, `time` and `locator` among them. The response echoes the identifiers and carries `authorized`, `evaluatedAt`, the optional `asOf` and `context`, and an optional `message` of at most 1024 characters. Neither direction carries the record's governance `context` or its validity window.

### Correlation

The query reveals which entity, authority, action and resource the relying party is interested in, and the document's `issuer` ties that interest to the relying party; carry it over an authenticated, confidential transport. `threadId` correlates request and response. A registry that answers anonymous queries learns nothing more than the identifiers asked about.

### Retention

The task is a read and persists nothing at the registry. The registry SHOULD NOT retain a log of queries beyond what its operational and abuse-prevention policy needs. Because the answer holds for one instant, consumers SHOULD NOT cache it beyond the record's validity window or the authority's stated caching policy.

### Consent/purpose

The purpose is answering whether the authority's statement about the entity holds at the evaluation instant. The registry SHOULD apply its own access-control policy to who may query.
