---
slug: did-management/did/rollback
version: "0.1"
title: DID Management — Rollback
summary: A DID owner reverts the slot's log chain to a prior published version, discarding the entries above that point.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, rollback, revert]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Rollback is destructive (entries above the target are discarded); the maintainer SHOULD retain a signed record of who authorised the revert and at what point in the chain.
errorCodes:
  - code: did-management/did/rollback:not_owner
    meaning: The caller is not the slot's owner or an admin.
    retryable: false
  - code: did-management/did/rollback:invalid_target_version
    meaning: The submitted `targetVersion` is greater than the current `versionCount` or less than 1.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/publish]
---

## Abstract

The **DID Management — Rollback** Trust Task discards log entries newer than `targetVersion`, returning the chain to that point. Destructive — there is no rollback-of-rollback. The slot's owner uses this to recover from a mistaken publish or a key compromise that signed entries the owner doesn't want carried forward.

## Status of this Document

Draft.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/rollback/0.1` with `payload.mnemonic` and `payload.targetVersion`. The consumer verifies ownership, validates the target version is in range, trims the chain, and returns the updated record plus the count of entries removed.

## Request

```json
{ "id": "rb-1", "type": "https://trusttasks.org/spec/did-management/did/rollback/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-03T10:00:00Z",
  "payload": { "mnemonic": "alice", "targetVersion": 3 } }
```

## Response

```json
{ "id": "rb-1-r", "type": "https://trusttasks.org/spec/did-management/did/rollback/0.1#response",
  "threadId": "rb-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-03T10:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-03T10:00:01Z",
    "versionCount": 3, "domain": "did.example.com", "disabled": false }, "removedVersions": 2 } }
```

## Security & Privacy

Rollback is a destructive operation. Consumers MUST require freshness on the producer's session and SHOULD audit every rollback at higher severity than ordinary publishes.
