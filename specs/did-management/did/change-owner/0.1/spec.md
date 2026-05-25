---
slug: did-management/did/change-owner
version: "0.1"
title: DID Management — Change Owner
summary: The current owner of a DID slot transfers ownership to a different VID.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, change-owner, transfer]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Current DID owner
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: A change-owner irrevocably removes the prior owner's authority on the slot. The maintainer SHOULD retain a signed evidentiary record.
errorCodes:
  - code: did-management/did/change-owner:not_owner
    meaning: The caller is not the slot's current owner.
    retryable: false
  - code: did-management/did/change-owner:target_not_authorized
    meaning: The proposed `newOwner` VID is not permitted by the host's ACL / domain scope.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/register]
---

## Abstract

The **DID Management — Change Owner** Trust Task transfers a DID slot's ownership from the current owner to a new VID. The transfer is one-shot — the prior owner immediately loses authority to mutate the slot, and the new owner becomes responsible for future publishes, disables, and ultimately deletion.

## Status of this Document

Draft.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/change-owner/0.1` with `payload.mnemonic` and `payload.newOwner`. The consumer verifies the caller is the current owner, verifies `newOwner` is authorised under the host's policy, and commits the ownership swap atomically.

## Request

```json
{ "id": "co-1", "type": "https://trusttasks.org/spec/did-management/did/change-owner/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-04T10:00:00Z",
  "payload": { "mnemonic": "alice", "newOwner": "did:key:z6MkBob" } }
```

## Response

```json
{ "id": "co-1-r", "type": "https://trusttasks.org/spec/did-management/did/change-owner/0.1#response",
  "threadId": "co-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-04T10:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkBob",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-04T10:00:01Z",
    "versionCount": 1, "domain": "did.example.com", "disabled": false } } }
```

## Security & Privacy

A change-owner is *irreversible without the cooperation of the new owner*. The signed document is the audit anchor; consumers MUST retain it.
