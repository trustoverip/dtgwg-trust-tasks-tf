---
slug: did-management/did/list
version: "0.1"
title: DID Management — List
summary: A caller enumerates DID slots they can see — their own when non-admin, or any/by-owner when admin.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, list, enumerate]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Querying party
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: List is read-only; integrity is usually carried by the transport.
errorCodes:
  - code: did-management/did/list:forbidden
    meaning: Non-admin caller specified an `owner` other than themselves.
    retryable: false
  - code: did-management:unknown_domain
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

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/list/0.1`. The consumer enforces scoping per the caller's role and returns a `records` array plus a `total` count.

## Request

```json
{
  "id": "list-req-1",
  "type": "https://trusttasks.org/spec/did-management/did/list/0.1",
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
  "type": "https://trusttasks.org/spec/did-management/did/list/0.1#response",
  "threadId": "list-req-1",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T12:00:01Z",
  "payload": {
    "records": [{
      "mnemonic": "alice", "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T10:05:01Z",
      "versionCount": 1, "method": "webvh", "domain": "did.example.com", "disabled": false
    }],
    "total": 1
  }
}
```

## Security & Privacy

Consumers MUST reject admin-scope requests from non-admin callers with `did-management/did/list:forbidden`. List results are typically not privacy-sensitive on public hosting domains; on private domains, rate-limit to slow enumeration.
