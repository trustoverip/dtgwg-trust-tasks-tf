---
slug: did-management/did/list
version: "0.2"
title: DID Management — List
summary: A caller enumerates DID slots they can see — their own when non-admin, or any/by-owner when admin.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords: [did, did-hosting, list, enumerate]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Querying party
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: List is read-only; integrity is usually carried by the transport.
sideEffects:
  level: none
  rationale: Read-only enumeration. Returns slots the caller may see and persists nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: did-management/did/list:forbidden
    meaning: Non-admin caller specified an `owner` other than themselves.
    retryable: false
  - code: did-management:unknownDomain
    meaning: The submitted `domain` filter is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related:
  - did-management/did/info
  - did-management/did/register
---

## Abstract

The **DID Management — List** Trust Task returns the set of DID slots visible to the caller. Non-admins see only slots they own; admins MAY pass `owner` to list a specific party's slots, or omit it to list every slot on the host. An optional `domain` filter restricts the result to slots hosted under that domain.

## Status of this Document

This is a **draft** *Trust Task specification*; the schema **MAY** change without notice.

## Changes from 0.1

Paging moves from offset to cursor, per the registry-wide pagination convention (request `cursor` + `limit`, response `nextCursor`, absent on the last page):

- Request **`offset`** (integer) is removed; it is replaced by **`cursor`** (optional opaque string), the `nextCursor` of a previous response. A consumer **MUST NOT** accept a cursor it did not issue.
- Request **`limit`** is unchanged on the wire (optional integer, 1–1000).
- Response **`total`** (integer, required) is removed. The new response **`nextCursor`** (optional string) is present when more records remain and absent on the last page; it is never `null`.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/list/0.2`. The consumer enforces scoping per the caller's role and returns a `records` array, plus a `nextCursor` when more records remain (absent on the last page). A caller pages by passing `nextCursor` back as `cursor`.

## Request

```json
{
  "id": "list-req-1",
  "type": "https://trusttasks.org/spec/did-management/did/list/0.2",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T12:00:00Z",
  "payload": { "limit": 50, "domain": "did.example.com" }
}
```

## Response

```json
{
  "id": "list-resp-1",
  "type": "https://trusttasks.org/spec/did-management/did/list/0.2#response",
  "threadId": "list-req-1",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T12:00:01Z",
  "payload": {
    "records": [{
      "mnemonic": "alice", "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T10:05:01Z",
      "versionCount": 1, "method": "webvh", "domain": "did.example.com", "disabled": false
    }]
  }
}
```

## Security & Privacy

### Data carried

The request carries optional filters (`owner`, `domain`) and paging members
(`cursor`, `limit`). The response carries
[`DidRecord`](../../../_shared/0.1/did-record.schema.json)s: each names a slot's
mnemonic, its owner's identifier, timestamps, version count, DID method and
hosting domain, and may carry the DID, its URL, a disabled flag and a resolve
count. Non-admins see only slots they own. Consumers MUST reject admin-scope
requests from non-admin callers with `did-management/did/list:forbidden`. The
specification imposes no further minimisation rule.

### Correlation

A record joins a slot to its owner's identifier and to its creation and update
times, so an admin listing joins every owner on the host to the slots they
hold. List results are typically not privacy-sensitive on public hosting
domains; on private domains, rate-limit to slow enumeration. `threadId`
correlates request and response, and `nextCursor`/`cursor` link the pages of
one enumeration; a consumer **MUST NOT** accept a cursor it did not issue.

### Retention

The task is a read-only enumeration and persists nothing at the consumer. The
specification imposes no retention rule on the caller's copy of the response.

### Consent/purpose

The purpose is to let a caller see the DID slots visible to it — its own, or,
for an admin, any party's or every slot on the host. The specification states
no further purpose limitation.
