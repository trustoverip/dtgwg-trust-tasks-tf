---
slug: did-management/agent-name/list
version: "0.1"
title: DID Management — List Agent Names
summary: A DID owner reads the authoritative agent-name registry for a hosted DID — every name bound to it, including parked entries that the DID document itself cannot show.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, agent-name, list, parked]
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
  requirement: OPTIONAL
  rationale: A read-only listing consumed over an authenticated transport; the enumeration is a view of state whose evidentiary records are the mutations that produced it.
sideEffects:
  level: none
  rationale: "Read-only enumeration of the slot's name registry; no state changes."
subjectPath: /mnemonic
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: did-management/agent-name/list:notOwner
    meaning: The caller is not the DID slot's current owner or an admin.
    retryable: false
  - code: did-management/agent-name/list:notFound
    meaning: No slot exists under the submitted mnemonic.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain, or is inconsistent with the slot's recorded domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/agent-name/update, did-management/agent-name/remove, did-management/agent-name/check]
---

## Abstract

The **DID Management — List Agent Names** Trust Task reads a hosted DID's agent-name registry — the authoritative forward view of every name bound to the slot, **including parked entries**.

The registry is the only place a parked name is visible: parking works by dropping the claim from the document's `alsoKnownAs`, so a client reading the DID document alone cannot tell a parked name from one that was never bound — and so cannot offer to resume it. This task is what a management surface calls before offering "resume" (via [`agent-name/update`](../../update/0.1/spec.md) with `state: active`) or "release" (via [`agent-name/remove`](../../remove/0.1/spec.md)).

The response carries the slot's hosting domain alongside the entries, because a bare local part (`alice`) only means something within a domain (`example.com/@alice`). Entries carry the bare local part, a resolution flag (`enabled: false` ⇔ parked), and the binding's creation time.

## Status of this Document

Draft.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/agent-name/list/0.1` with `payload.mnemonic` and optional `payload.domain` (disambiguation only — when present it MUST match the slot's recorded domain, per [category conventions §3](../../../_shared/0.1/CONVENTIONS.md#3-per-domain-mnemonic-disambiguation)). The consumer verifies the caller is the slot's owner or an admin and returns every entry in the slot's registry. `agentNames` is always present — a slot with no names answers with an empty array.

## Request

```json
{ "id": "ls-1", "type": "https://trusttasks.org/spec/did-management/agent-name/list/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-07-20T09:00:00Z",
  "payload": { "mnemonic": "alice" } }
```

## Response

```json
{ "id": "ls-1-r", "type": "https://trusttasks.org/spec/did-management/agent-name/list/0.1#response",
  "threadId": "ls-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-07-20T09:00:01Z",
  "payload": { "mnemonic": "alice", "domain": "did.example.com",
    "agentNames": [
      { "name": "alice", "enabled": true, "createdAt": "2026-07-01T10:00:00Z" },
      { "name": "ally", "enabled": false, "createdAt": "2026-07-05T12:00:00Z" }
    ] } }
```

## Security & Privacy

The listing is owner-scoped, and that is the point of it: parked names are deliberately invisible to the public (they are absent from the document and the redirect surface), so this task is the one place they can be enumerated. A host MUST apply the same ownership check it applies to the mutating verbs — leaking a tenant's parked names reveals names they intend to reclaim, which is exactly the information a name-squatter wants.
