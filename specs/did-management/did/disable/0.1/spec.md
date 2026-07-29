---
slug: did-management/did/disable
version: "0.1"
title: DID Management — Disable
summary: A DID owner (or admin) suspends hosting for a DID without deleting it — resolution returns a deactivation marker but content is retained.
status: retired
supersededBy: did-management/did/set-state
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, disable, suspend]
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
  rationale: Disable is an evidentiary state transition; auditors retain the document to corroborate the change.
sideEffects:
  level: mutating
  rationale: "Suspends resolution while retaining content; reversible via did/enable."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/did/disable:not_owner
    meaning: The caller is not the slot's current owner or an admin.
    retryable: false
  - code: did-management/did/disable:already_disabled
    meaning: The slot is already in the disabled state.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/enable, did-management/did/delete]
---

## Abstract

The **DID Management — Disable** Trust Task suspends a hosted DID's public resolvability. The slot is marked `disabled` and the resolver serves a deactivation marker, but the underlying log and witness content remain on disk for recovery via [`did-management/did/enable`](../../enable/0.1/spec.md). Use Disable for temporary suspensions (key compromise investigations, billing holds); use [`did-management/did/delete`](../../delete/0.1/spec.md) for owner-initiated terminations.

## Status of this Document

**Retired.** This task is superseded by [`did-management/did/set-state`](../../../did/set-state/0.1/spec.md) with `state: suspended`. Consumers SHOULD NOT accept new documents of this type; the specification is retained for auditability of previously-issued documents.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/disable/0.1` with `payload.mnemonic` (and optional `payload.domain` per [category conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-domain-resolution)). The consumer verifies ownership, transitions the record, and returns the updated `DidRecord`.

## Request

```json
{ "id": "dis-1", "type": "https://trusttasks.org/spec/did-management/did/disable/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T14:00:00Z", "payload": { "mnemonic": "alice" } }
```

## Response

```json
{ "id": "dis-1-r", "type": "https://trusttasks.org/spec/did-management/did/disable/0.1#response",
  "threadId": "dis-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T14:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T14:00:01Z",
    "versionCount": 1, "domain": "did.example.com", "disabled": true } } }
```

## Security & Privacy

A captured disable document is evidence of suspension; the REQUIRED proof binds the action to the producer. Consumers MUST reject replays via `id` uniqueness.
