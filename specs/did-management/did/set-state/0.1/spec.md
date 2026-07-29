---
slug: did-management/did/set-state
version: "0.1"
title: DID Management — Set State
summary: A DID owner (or admin) sets a hosted DID's hosting state — `active` for live resolution, `suspended` to serve a deactivation marker while content is retained. Supersedes the separate enable / disable verbs.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did, did-hosting, state, suspend, restore, enable, disable]
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
  rationale: A hosting-state transition is evidentiary in both directions — suspension is a change auditors corroborate, and restoring resolution has real downstream consequences (resolution traffic resumes, witness watchers re-engage) — so the maintainer retains a signed record.
sideEffects:
  level: mutating
  rationale: "Moves a DID slot between live resolution and suspension. Both directions are reversible with another set-state; content is retained throughout."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/did/set-state:notOwner
    meaning: The caller is not the slot's current owner or an admin.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/did/delete, did-management/did/register]
---

## Abstract

The **DID Management — Set State** Trust Task sets a hosted DID's public hosting state.

- **`state: active`** — the slot resolves normally (the old [`did/enable`](../../enable/0.1/spec.md)).
- **`state: suspended`** — the slot is marked disabled and the resolver serves a deactivation marker, but the underlying log and witness content remain on disk for recovery (the old [`did/disable`](../../disable/0.1/spec.md)).

The two retired verbs had identical payloads and were each other's inverse; this task replaces the pair with one declarative state field. Requesting the state the slot is already in succeeds as an idempotent no-op — the `alreadyDisabled` / `notDisabled` error class of the verb pair does not exist here, because a declarative request for the current state is not a fault.

Use `suspended` for temporary suspensions (key-compromise investigations, billing holds); use [`did-management/did/delete`](../../delete/0.1/spec.md) for owner-initiated terminations — deletion is a distinct lifecycle (recovery-window soft-delete), not a third state of this task.

## Status of this Document

Draft per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); schema **MAY** change without notice.

## Conformance

The producer emits `type: https://trusttasks.org/spec/did-management/did/set-state/0.1` with `payload.mnemonic` and `payload.state` (and optional `payload.domain` per [category conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-domain-resolution)). The consumer verifies ownership, transitions the record to the requested state (a no-op when already there), and returns the updated `DidRecord`, whose `disabled` field reflects the result (`true` ⇔ `suspended`).

## Request

```json
{ "id": "st-1", "type": "https://trusttasks.org/spec/did-management/did/set-state/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T14:00:00Z", "payload": { "mnemonic": "alice", "state": "suspended" } }
```

## Response

```json
{ "id": "st-1-r", "type": "https://trusttasks.org/spec/did-management/did/set-state/0.1#response",
  "threadId": "st-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T14:00:01Z",
  "payload": { "record": { "mnemonic": "alice", "owner": "did:key:z6MkAlice",
    "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T14:00:01Z",
    "versionCount": 1, "domain": "did.example.com", "disabled": true } } }
```

## Security & Privacy

A captured set-state document is evidence of the transition; the REQUIRED proof binds the action to the producer. Consumers MUST reject replays via `id` uniqueness — a replayed `suspended` after a legitimate `active` (or vice versa) silently undoes the owner's latest intent. Restoring `active` resumes public resolvability — consumers MUST audit the document and bind the change to a fresh-enough authenticated session.
