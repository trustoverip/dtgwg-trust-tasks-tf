---
slug: did-management/domain/set-state
version: "0.1"
title: DID Management — Domain Set State
summary: An administrator sets a hosting domain's service state — `active` restores it to service (cancelling any pending purge), `disabled` moves it to read-only with a purge grace period. Supersedes the separate enable / disable verbs.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, state, enable, disable, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A domain state transition has a fleet-wide effect in either direction; the maintainer SHOULD retain a signed record.
sideEffects:
  level: mutating
  rationale: "Moves a hosting domain between active service and read-only disablement with a grace period. Both directions are reversible with another set-state until the purge grace period expires."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/set-state:isDefault
    meaning: "`state: disabled` was requested for the system default domain; the default must first be moved elsewhere via `domain/set-default`."
    retryable: false
  - code: did-management/domain/set-state:alreadyPurged
    meaning: "`state: active` was requested for a domain already purged after its grace period expired; it can no longer be re-activated and must be recreated via `domain/create`."
    retryable: false
related: [did-management/domain/set-default, did-management/domain/purge, did-management/domain/create]
---

## Abstract

The **Domain Set State** Trust Task sets a hosting domain's service state.

- **`state: active`** — the domain is in active service: new DID registrations resume, and — when applied to a disabled domain — the `disabledAt` / `purgeAt` markers are cleared and any pending purge is cancelled (the old [`domain/enable`](../../enable/0.1/spec.md)).
- **`state: disabled`** — the domain moves to read-only: existing slots remain resolvable until the host's purge grace period expires, new registrations are rejected, and `disabledAt` / `purgeAt` are stamped (the old [`domain/disable`](../../disable/0.1/spec.md)).

The two retired verbs were mirror-images over the same single-field payload; this task replaces the pair with one declarative state field. Requesting the state the domain is already in succeeds as an idempotent no-op and returns the current entry — the `alreadyDisabled` error of the verb pair does not exist here.

Two boundaries are **not** states of this task: the system default domain cannot be disabled until the default is moved via [`domain/set-default`](../../set-default/0.1/spec.md) (`isDefault`), and a domain already purged after its grace period cannot be re-activated — it must be recreated via [`domain/create`](../../create/0.1/spec.md) (`alreadyPurged`).

A successful transition triggers the same fleet-wide fan-out as create/update: the control plane pushes the updated `DomainEntry` to every registered server so their resolvers see the new state without waiting for a refresh.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/set-state/0.1` with `payload.name` and `payload.state`. Consumer rejects with `did-management:unknown_domain` if the domain is not in the registry; with `isDefault` when disabling the current system default; and with `alreadyPurged` when activating a purged domain. Otherwise it transitions the entry (a no-op when already in the requested state), fans the updated entry out to registered servers, and replies with the updated `DomainEntry`.

## Request

```json
{ "id": "ds-1", "type": "https://trusttasks.org/spec/did-management/domain/set-state/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-12T09:00:00Z", "payload": { "name": "tenant-a.example.com", "state": "disabled" } }
```

## Response

```json
{ "id": "ds-1-r", "type": "https://trusttasks.org/spec/did-management/domain/set-state/0.1#response",
  "threadId": "ds-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-12T09:00:01Z",
  "payload": { "entry": { "name": "tenant-a.example.com", "label": "Tenant A",
    "status": "disabled", "defaultDomain": false, "createdAt": "2026-06-10T09:00:01Z",
    "disabledAt": "2026-06-12T09:00:01Z", "purgeAt": "2026-06-19T09:00:01Z" } } }
```

The response carries the canonical `DomainEntry` after the transition — `disabledAt` and `purgeAt` are absent on an active entry.

## Security & Privacy

Admin-only in both directions. Disabling cascades to read-only across every active DID on the domain — the caller must understand the operational scope. Re-activating restores the path-namespace the domain owned before disablement; consumers MUST ensure the caller still holds admin authority on the domain and that no conflicting `domain/create` has bound the same `name` to a different entry during the grace window. Consumers MUST reject replays via `id` uniqueness — a replayed transition silently undoes the operator's latest intent.
