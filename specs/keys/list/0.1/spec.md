---
slug: keys/list
version: "0.1"
title: Keys — List
summary: A producer enumerates the keys a custodian holds, optionally filtered by lifecycle state or scope, with the total so a partial page is never mistaken for the whole set.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - list
  - enumerate
  - inventory
  - pagination
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: An inventory read is usually consumed over an authenticated transport; a proof matters when the listing is retained as evidence of what a custodian held at a point in time.
sideEffects:
  level: none
  rationale: "Read-only enumeration."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - keys/show
  - keys/create
  - keys/revoke
---

## Abstract

The **Keys — List** Trust Task enumerates the key records a *key custodian* holds, optionally narrowed to a lifecycle `status` or a `contextId`. It is the inventory surface behind key audits and rotation sweeps.

The response carries `total` alongside the page. That member is not decoration: a rotation sweep that reads one short page and stops has silently skipped every key past the page boundary, and without `total` nothing in the response distinguishes "this is all of them" from "this is the first twenty".

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/list/0.1`, with itself as `issuer` and the custodian as `recipient`. Every payload member is an optional filter; an empty payload requests everything the producer may see.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Return only records this producer is permitted to see. Filtering by policy is **not** an error — the producer receives a smaller set, and `total` reflects that filtered set, not the custodian's full inventory.
3. Populate `total`, `offset` and `limit` on every response, and set `total` to the number of records matching the request's filters *after* policy filtering.
4. **Not** include private key material.

A producer that intends to act on every key **MUST** page until it has seen `total` records; a custodian **MAY** return fewer than the requested `limit` and **MUST** report the applied value.

## Definitions

* **Producer.** The party enumerating; identified by `issuer`.
* **Key custodian.** The party holding the keys; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/list/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Active keys in one scope

```json
{
  "id": "60718293-a4b5-4c62-d7e8-f90112233445",
  "type": "https://trusttasks.org/spec/keys/list/0.1",
  "issuer": "did:web:auditor.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:30:00Z",
  "payload": {
    "status": "active",
    "contextId": "app",
    "limit": 2
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/list/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

Note `total: 5` against two returned records — the producer has four more pages' worth to read before it has seen the scope:

```json
{
  "id": "718293a4-b5c6-4d72-e8f9-011223344556",
  "type": "https://trusttasks.org/spec/keys/list/0.1#response",
  "threadId": "60718293-a4b5-4c62-d7e8-f90112233445",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:auditor.example",
  "issuedAt": "2026-07-31T09:30:01Z",
  "payload": {
    "keys": [
      {
        "keyId": "app-signing-key",
        "keyType": "ed25519",
        "status": "active",
        "publicKey": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "origin": "derived",
        "contextId": "app",
        "createdAt": "2026-07-31T09:20:01Z",
        "updatedAt": "2026-07-31T09:20:01Z"
      },
      {
        "keyId": "app-agreement-key",
        "keyType": "x25519",
        "status": "active",
        "publicKey": "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc",
        "origin": "derived",
        "contextId": "app",
        "createdAt": "2026-07-31T09:21:00Z",
        "updatedAt": "2026-07-31T09:21:00Z"
      }
    ],
    "total": 5,
    "offset": 0,
    "limit": 2
  }
}
```

## Security & Privacy

A full listing is a map of the custodian's key set: what exists, what it is for, and what is still usable. Custodians **SHOULD** return only what the producer is entitled to see and **SHOULD NOT** treat an unfiltered request as a request for everything.

Omitting `status` returns revoked keys as well as active ones, which is usually what an auditor wants and rarely what an application wants. Producers that mean "keys I could sign with" **MUST** filter on `status: "active"` rather than assuming the default excludes revoked material.
