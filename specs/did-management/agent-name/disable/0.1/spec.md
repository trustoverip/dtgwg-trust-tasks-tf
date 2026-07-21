---
slug: did-management/agent-name/disable
version: "0.1"
title: DID Management — Disable Agent Name
summary: Park an agent name — stop it resolving while keeping it reserved to this DID. The submitted DID document must no longer claim it via `alsoKnownAs`.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, alsoKnownAs, suspend]
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
  rationale: Disable is an evidentiary state transition; auditors retain the document to corroborate that the name stopped resolving.
sideEffects:
  level: mutating
  rationale: "Stops a name resolving while keeping its reservation; reversible via agent-name/enable."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/disable:not_owner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/disable:not_found
    meaning: No such name is bound to this DID.
    retryable: false
  - code: did-management/agent-name/disable:already_disabled
    meaning: The name is already in the disabled state.
    retryable: false
  - code: did-management/agent-name/disable:also_known_as_mismatch
    meaning: The submitted `didData` still claims the name via `alsoKnownAs`. Parking a name MUST be accompanied by a document that no longer claims it.
    retryable: false
  - code: did-management/agent-name/disable:invalid_did_data
    meaning: The submitted `didData` failed proof or structural validation for the target DID.
    retryable: false
  - code: did-management/agent-name/disable:step_up_required
    meaning: The operation requires a higher authentication assurance level (operator step-up) that has not been satisfied. Suspending a name is a consumer-gated operation.
    retryable: true
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/enable, did-management/agent-name/remove, auth/step-up/approve-response]
---

## Abstract

The **DID Management — Disable Agent Name** Trust Task **parks** an agent name: it stops resolving, but the host keeps it reserved so no other DID can claim it. Use disable for a temporary suspension you intend to reverse with [`agent-name/enable`](../../enable/0.1/spec.md); use [`agent-name/remove`](../../remove/0.1/spec.md) to release the name entirely.

## Status of this Document

Draft.

## The `alsoKnownAs` invariant

A parked name must stop resolving, so the submitted `didData` must **no longer** claim it via `alsoKnownAs` — the host rejects with `also_known_as_mismatch` otherwise. The distinction from remove is entirely host-side: both submit a document that drops the name, but disable **keeps the reservation** in the host's registry while remove frees it. The document cannot express a reservation, which is why parking is a hosting-service concept layered on top of the `alsoKnownAs` state, not derivable from the document alone.

## Step-up

Suspending a name takes it out of service, which can disrupt anyone relying on it. A consumer **MUST** gate this task behind operator **step-up** (see [`auth/step-up/approve-response`](../../../../auth/step-up/approve-response/0.1/spec.md)) and, until satisfied, respond with `step_up_required`. As with all such requirements this is a **consumer policy** derived from the operation's impact, not a registry-enforceable flag.

## Request

```json
{ "id": "dis-1", "type": "https://trusttasks.org/spec/did-management/agent-name/disable/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs no longer contains example.com/@alice>" } }
```

## Response

```json
{ "id": "dis-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/disable/0.1#response",
  "threadId": "dis-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-20T09:00:01Z",
    "versionCount": 5, "domain": "did.example.com", "disabled": false } } }
```

Note `record.disabled` refers to the **DID slot**, not the name — a DID with an active slot can carry a disabled name. Per-name state is not surfaced on the shared `DidRecord`; a caller reads it back via a listing.

## Security & Privacy

A parked name remains reserved indefinitely. A host applying quotas SHOULD count parked names against a tenant's allocation, since they occupy the namespace.
