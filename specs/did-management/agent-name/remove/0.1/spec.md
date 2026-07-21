---
slug: did-management/agent-name/remove
version: "0.1"
title: DID Management — Remove Agent Name
summary: Release an agent name from a hosted DID. The name stops resolving and becomes claimable by anyone. The submitted DID document must no longer claim it via `alsoKnownAs`.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, alsoKnownAs, release]
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
  rationale: Releasing a name is an evidentiary transition an auditor retains — the name leaves this DID's control and may be re-registered by another party.
sideEffects:
  level: destructive
  rationale: "The name stops resolving and its reservation is released, so it can be claimed by a different DID. Not reversible by re-issuing the same request — reclaiming requires the name still be free."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/remove:not_owner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/remove:not_found
    meaning: No such name is bound to this DID.
    retryable: false
  - code: did-management/agent-name/remove:also_known_as_mismatch
    meaning: The submitted `didData` still claims the name via `alsoKnownAs`. A release MUST be accompanied by a document that no longer claims the name, so the two states cannot diverge.
    retryable: false
  - code: did-management/agent-name/remove:invalid_did_data
    meaning: The submitted `didData` failed proof or structural validation for the target DID.
    retryable: false
  - code: did-management/agent-name/remove:step_up_required
    meaning: The operation requires a higher authentication assurance level (operator step-up) that has not been satisfied. Releasing a name is destructive and a consumer MUST gate it behind step-up.
    retryable: true
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/set, did-management/agent-name/disable, auth/step-up/approve-response]
---

## Abstract

The **DID Management — Remove Agent Name** Trust Task releases an agent name from a hosted DID. Unlike [`agent-name/disable`](../../disable/0.1/spec.md) — which parks a name while keeping it reserved — remove **frees the name entirely**: it stops resolving and becomes claimable by any other DID on the host.

## Status of this Document

Draft.

## The `alsoKnownAs` invariant, in reverse

[`agent-name/set`](../../set/0.1/spec.md) requires the submitted document to *claim* the name. Remove requires the mirror: the submitted `didData` must **no longer** claim the name via `alsoKnownAs`. The host verifies this and rejects with `also_known_as_mismatch` otherwise.

The reason is symmetry of the invariant. If the host stopped serving a name while the document still claimed it, the DID would advertise an `alsoKnownAs` entry that no longer resolves — a dangling claim a resolver cannot verify. So the release of the redirect and the removal of the `alsoKnownAs` entry land in one commit, driven by one signed document.

## Step-up

Releasing a name is **destructive**: once free, the name may be registered by a different party, and the previous holder cannot unilaterally get it back. A consumer **MUST** gate this task behind operator **step-up** — an elevated authentication assurance level satisfied by an operator-signed approval (see [`auth/step-up/approve-response`](../../../../auth/step-up/approve-response/0.1/spec.md)) — and, until it is satisfied, respond with `step_up_required`.

Per the framework, this requirement is a **consumer policy** derived from the task's destructive classification; it is not, and cannot be, delegated to the registry. The declaration here is descriptive guidance, not a wire-enforceable flag.

## Request

```json
{ "id": "rm-1", "type": "https://trusttasks.org/spec/did-management/agent-name/remove/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice", "name": "alice",
    "didData": "<jsonl: a new signed log entry whose document alsoKnownAs no longer contains example.com/@alice>" } }
```

## Response

```json
{ "id": "rm-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/remove/0.1#response",
  "threadId": "rm-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-20T09:00:01Z",
    "versionCount": 3, "domain": "did.example.com", "disabled": false } } }
```

## Security & Privacy

A released name carries no memory of its former holder. An operator who wants to retire a name **without** letting anyone else take it should use [`agent-name/disable`](../../disable/0.1/spec.md), which keeps the reservation.
