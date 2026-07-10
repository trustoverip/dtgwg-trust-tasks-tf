---
slug: did-management/did/delete
version: "0.1"
title: DID Management — Delete
summary: A DID owner soft-deletes a hosted DID — the slot is taken off the public resolution path but content is retained for the host's recovery window.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, delete, soft-delete]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A delete is an evidentiary record of authorised removal; auditors retain the document to corroborate the action.
sideEffects:
  level: destructive
  rationale: Takes the DID off the public resolution path. Reversible only within the host's recovery window, after which resolution fails permanently — a human is consenting to making the identity unresolvable.
consequences:
  - The DID stops resolving for anyone relying on it, effective immediately.
  - Recovery is possible only inside the host's retention window; after that it is permanent.
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/did/delete:not_owner
    meaning: The caller is not the slot's current owner (and is not an admin).
    retryable: false
  - code: did-management/did/delete:already_deleted
    meaning: The slot is already in the deleted state.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/disable, did-management/did/register]
---

## Abstract

The **DID Management — Delete** Trust Task soft-deletes a hosted DID. The slot is marked deleted (`record.disabled = true` and `deletedAt` set), public resolution returns a deactivation marker, and content is preserved for the host's recovery window before the background sweep eligibility kicks in. Hard deletion is the host's responsibility — not exposed as a Trust Task here.

## Status of this Document

Draft per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); schema **MAY** change without notice.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/delete/0.1` with `payload.mnemonic`. The consumer verifies ownership, transitions the record, and returns the updated `DidRecord`.

## Request

```json
{ "id": "del-1", "type": "https://trusttasks.org/spec/did-management/did/delete/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T13:00:00Z", "payload": { "mnemonic": "alice" } }
```

## Response

```json
{ "id": "del-1-r", "type": "https://trusttasks.org/spec/did-management/did/delete/0.1#response",
  "threadId": "del-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T13:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T13:00:01Z",
    "versionCount": 1, "domain": "did.example.com", "disabled": true } } }
```

## Security & Privacy

Soft-delete preserves content for recovery; a true "forget me" guarantee requires a follow-up out-of-band purge by the host. Consumers MUST ensure deletion intent is auditable.
