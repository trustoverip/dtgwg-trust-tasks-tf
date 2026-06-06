---
slug: did-management/did/info
version: "0.1"
title: DID Management — Info
summary: A caller reads the canonical metadata record the hosting service holds for a single DID slot.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - did
  - did-hosting
  - info
  - read
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
  rationale: A single-record lookup is typically short-lived and consumed over an authenticated transport; a proof becomes valuable when the answer is retained or relied on by a third party.
errorCodes:
  - code: did-management/did/info:not_found
    meaning: No record exists for the requested mnemonic.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related:
  - did-management/did/list
  - did-management/did/register
---

## Abstract

The **DID Management — Info** Trust Task is a read-only query: given a `mnemonic`, return the `DidRecord` the hosting service currently holds, plus an optional summary of the latest log entry. Non-owners are permitted to query slots they don't own — the response is the *public* view of the record (i.e. omits any fields the consumer treats as host-internal).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/info/0.1` with `payload.mnemonic`. The consumer validates per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements), looks up the record, and either returns it (success) or replies with `did-management/did/info:not_found`.

## Definitions

* **DidRecord** — see [`_shared/0.1/did-record.schema.json`](../../../_shared/0.1/did-record.schema.json).
* **logSummary** — optional `{ latestVersionId, latestVersionTime }` block derived from the most recent log entry. The consumer MAY omit when the log has not been published yet (`versionCount: 0`).

## Request

```json
{
  "id": "info-req-1",
  "type": "https://trusttasks.org/spec/did-management/did/info/0.1",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T11:00:00Z",
  "payload": { "mnemonic": "alice" }
}
```

## Response

```json
{
  "id": "info-resp-1",
  "type": "https://trusttasks.org/spec/did-management/did/info/0.1#response",
  "threadId": "info-req-1",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T11:00:01Z",
  "payload": {
    "record": {
      "mnemonic": "alice", "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T10:05:01Z",
      "versionCount": 1, "didId": "did:webvh:abc:did.example.com:alice",
      "method": "webvh", "domain": "did.example.com", "disabled": false
    },
    "logSummary": { "latestVersionId": "1-abc", "latestVersionTime": "2026-06-01T10:04:59Z" }
  }
}
```

## Security & Privacy

Hosts on private hosting domains MAY rate-limit info queries to slow enumeration of mnemonic space; the response shape MUST NOT leak owner-private fields (e.g. retention timestamps) to non-owner callers.
