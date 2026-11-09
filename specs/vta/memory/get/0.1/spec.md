---
slug: vta/memory/get
version: "0.1"
title: VTA Memory — Get
summary: An agent recalls up to 256 named memory items from a VTA context in one round trip, each with its version, and learns which keys hold nothing.
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - memory
  - agent
  - recall
  - batch-read
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request changes no durable state, so the transport's authenticated sender
    is sufficient for it. The response hands the caller stored memory, and the
    SPEC §7.3 item 8 floor for a `discloses: secret` response applies.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The response returns stored memories, which are secret to their owner; the acceptance window bounds how long a captured request keeps returning them.
sideEffects:
  level: none
  rationale: "Read-only recall of named memory items."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns the values of the named items — free text the agent
    wrote. The request carries only the context and the keys the agent already
    knows.
retention:
  class: exchange
  rationale: The VTA remains the record of the memory; the response is a working copy for the session that asked for it.
errorCodes:
  - code: vta/memory/get:contextForbidden
    meaning: The caller is not permitted to read memory in the named context.
    retryable: false
  - code: vta/memory/get:duplicateKey
    meaning: The `keys` array contains the same key more than once. Refused rather than deduplicated, because a caller that sent a duplicate did not mean to.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        keys:
          type: array
          items:
            type: string
related:
  - vta/memory/list
  - vta/memory/put
  - vta/memory/delete
  - vta/app-state/get-many
---

## Abstract

The **VTA Memory — Get** Trust Task recalls named memory items from one VTA
context: the read an agent makes when it knows what it needs. It takes up to
256 keys and answers each one exactly once — found (`items`, with its value and
version), absent (`missing`), or not evaluated because the response reached its
size budget (`deferred`) — so the agent can account for every key it asked
about.

Each returned item carries its `version`, which the agent passes as
`expectedVersion` to [`vta/memory/put`](../../put/0.2/spec.md) or
[`vta/memory/delete`](../../delete/0.2/spec.md) to make a read-modify-write safe.

Because the request names its keys and the response says which it returned,
this is also the read whose audit record can say exactly which items an agent
saw — the question a principal asks of their agents' memory.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/get/0.1` with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` and `payload.keys`, with no key twice.
3. Re-request the keys in `deferred` if it still needs them.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements), and sign the response.
2. Refuse with `vta/memory/get:contextForbidden` when the caller lacks read access to memory in `payload.contextId`.
3. Refuse a request naming one key twice with `vta/memory/get:duplicateKey`, listing the duplicates.
4. Place every requested key in exactly one of `items`, `missing` or `deferred`. A key holding a tombstone goes to `missing`, or to `items` as a `deleted: true` record when `includeDeleted` is true. `items` follow the order of `keys`.
5. Return each found item's `value` and `version`.
6. When the response would exceed its size budget, stop evaluating and put the remaining keys in `deferred`, rather than refuse the request. At least one key **MUST** be evaluated, since a single item never exceeds the per-item cap of `vta/memory/put`.

## Authorization

Authority is **read access to memory in the named context**, the same grant
[`vta/memory/list`](../../list/0.2/spec.md) requires. Naming a key confers
nothing: a caller without read access to the context is refused whichever keys
it names, and is not told which of them exist.

## Request

```json
{
  "id": "a06c7b3f-5d99-4e76-a165-8f2d7a1c9e45",
  "type": "https://trusttasks.org/spec/vta/memory/get/0.1",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T15:00:00Z",
  "payload": {
    "contextId": "finance",
    "keys": ["preference/invoice-format", "contact/billing", "contact/escalation"]
  }
}
```

## Response

```json
{
  "id": "d5fc7b4b-0a6e-4b5d-8e8f-4a1b9c3d6ea5",
  "type": "https://trusttasks.org/spec/vta/memory/get/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-11-09T15:00:01Z",
  "threadId": "a06c7b3f-5d99-4e76-a165-8f2d7a1c9e45",
  "payload": {
    "items": [
      {
        "contextId": "finance",
        "key": "preference/invoice-format",
        "version": 55,
        "value": "PDF, one file per invoice",
        "valueBytes": 25,
        "deleted": false,
        "createdAt": "2026-11-09T13:30:00Z",
        "updatedAt": "2026-11-09T13:30:00Z"
      }
    ],
    "missing": ["contact/billing", "contact/escalation"]
  }
}
```

`contact/billing` holds a tombstone and is reported missing because
`includeDeleted` was not set.

## Security & Privacy

**`missing` answers a question about existence.** It tells a caller with read
access which keys hold nothing — which that caller could learn from
`vta/memory/list` anyway. It is never sent to a caller without read access,
which is refused before any key is looked up.

### Data carried

The request carries the context and up to 256 keys the agent already knows. The
response carries the values of the items found, and the keys of those absent or
deferred.

### Correlation

A get names exactly what the agent was interested in, which a list does not, so
a VTA's record of gets is a sharper account of an agent's attention than its
record of lists. That is intended — it is what makes the audit below useful —
and is a reason to keep that record on the principal's side.

### Retention

The VTA keeps nothing new because of a get. **Audit.** A VTA **SHOULD** record
each get with the requesting DID, the context, and the keys in `items` — the
keys actually returned, not those requested, and never a `deferred` key, which
was not read — so that "which agent saw what" can be answered per item. It
**SHOULD NOT** copy values into the audit trail.

### Consent/purpose

The purpose is recall inside one context. A VTA **MUST NOT** return an item
written in one context to a caller scoped to another.
