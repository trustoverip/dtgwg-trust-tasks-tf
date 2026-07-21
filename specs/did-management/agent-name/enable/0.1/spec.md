---
slug: did-management/agent-name/enable
version: "0.1"
title: DID Management — Enable Agent Name
summary: Return a parked (disabled) agent name to live resolution. The submitted DID document must claim it again via `alsoKnownAs`.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, alsoKnownAs, restore]
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
  rationale: Re-enabling resumes public resolvability; auditors retain the signed document to corroborate the change.
sideEffects:
  level: mutating
  rationale: "Returns a parked name to live resolution; reversible via agent-name/disable."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/enable:not_owner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/enable:not_found
    meaning: No such name is bound to this DID.
    retryable: false
  - code: did-management/agent-name/enable:not_disabled
    meaning: The name is not in the disabled state; nothing to re-enable.
    retryable: false
  - code: did-management/agent-name/enable:also_known_as_mismatch
    meaning: The submitted `didData` does not claim the name via `alsoKnownAs`. A host MUST NOT resume serving a name the document does not claim.
    retryable: false
  - code: did-management/agent-name/enable:invalid_did_data
    meaning: The submitted `didData` failed proof or structural validation for the target DID.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/disable, did-management/agent-name/set]
---

## Abstract

The **DID Management — Enable Agent Name** Trust Task returns a **parked** agent name to live resolution. It is the inverse of [`agent-name/disable`](../../disable/0.1/spec.md). Rejected for a name that is already active (`not_disabled`) or that was never bound (`not_found`).

## Status of this Document

Draft.

## The `alsoKnownAs` invariant

Disabling a name removes it from the document's `alsoKnownAs` (so it stops resolving) while keeping the host's reservation. Enabling must restore both halves: the submitted `didData` must claim the name again, and the host resumes serving the redirect in the same commit. A host rejects with `also_known_as_mismatch` if the document does not claim the name, because a served name whose document is silent about it fails Layer-1 verification at every resolver.

## Request

```json
{ "id": "en-1", "type": "https://trusttasks.org/spec/did-management/agent-name/enable/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs contains example.com/@alice>" } }
```

## Response

```json
{ "id": "en-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/enable/0.1#response",
  "threadId": "en-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-20T09:00:01Z",
    "versionCount": 4, "domain": "did.example.com", "disabled": false } } }
```

## Security & Privacy

Enable resumes public resolvability — a consumer binds the change to a fresh-enough authenticated session and audits the submitted document before serving.
