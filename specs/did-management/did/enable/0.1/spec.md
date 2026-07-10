---
slug: did-management/did/enable
version: "0.1"
title: DID Management — Enable
summary: Inverse of `did/disable` — a previously suspended DID slot is returned to live resolution.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, enable, restore]
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
  rationale: Enable is an evidentiary state transition; restoring a suspended DID has real downstream consequences (resolution traffic resumes, witness watchers re-engage) so the maintainer SHOULD retain a signed record of the change.
sideEffects:
  level: mutating
  rationale: "Returns a suspended DID slot to live resolution."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/did/enable:not_owner
    meaning: The caller is not the slot's current owner or an admin.
    retryable: false
  - code: did-management/did/enable:not_disabled
    meaning: The slot is not in the disabled state; nothing to re-enable.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/disable]
---

## Abstract

The **DID Management — Enable** Trust Task transitions a disabled DID slot back to live resolution. Idempotent for an already-active slot; rejected for a slot that's never been disabled.

## Status of this Document

Draft.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/enable/0.1` with `payload.mnemonic` (and optional `payload.domain`). The consumer clears the `disabled` flag and returns the updated record.

## Request

```json
{ "id": "ena-1", "type": "https://trusttasks.org/spec/did-management/did/enable/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-02T09:00:00Z", "payload": { "mnemonic": "alice" } }
```

## Response

```json
{ "id": "ena-1-r", "type": "https://trusttasks.org/spec/did-management/did/enable/0.1#response",
  "threadId": "ena-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-02T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-02T09:00:01Z",
    "versionCount": 1, "domain": "did.example.com", "disabled": false } } }
```

## Security & Privacy

Enable resumes public resolvability — consumers MUST audit the document and bind it to a fresh-enough authenticated session.
