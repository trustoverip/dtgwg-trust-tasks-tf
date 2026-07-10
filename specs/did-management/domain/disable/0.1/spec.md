---
slug: did-management/domain/disable
version: "0.1"
title: DID Management — Domain Disable
summary: An administrator disables a hosting domain — existing DIDs remain readable for the host's grace period, but no new DIDs may be hosted under it.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, disable, admin]
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
  rationale: Disabling a domain has a fleet-wide effect; the maintainer SHOULD retain a signed record.
sideEffects:
  level: mutating
  rationale: "Disables a hosting domain with a grace period; reversible via enable."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/disable:is_default
    meaning: The domain is the system default; cannot disable until the default is moved elsewhere via `domain/set-default`.
    retryable: false
  - code: did-management/domain/disable:already_disabled
    meaning: The domain is already in the disabled state.
    retryable: false
related: [did-management/domain/set-default, did-management/domain/purge]
---

## Abstract

The **Domain Disable** Trust Task moves a hosting domain into read-only mode. Existing slots remain resolvable until the host's purge grace period expires; new registrations are rejected. The system default cannot be disabled — operators must move the default first via [`domain/set-default`](../../set-default/0.1/spec.md).

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/disable/0.1` with `payload.name`. Consumer rejects if the domain is the current system default; otherwise transitions the entry and stamps `disabledAt` and `purgeAt`.

## Request

```json
{ "id": "dd-1", "type": "https://trusttasks.org/spec/did-management/domain/disable/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-12T09:00:00Z", "payload": { "name": "tenant-a.example.com" } }
```

## Response

```json
{ "id": "dd-1-r", "type": "https://trusttasks.org/spec/did-management/domain/disable/0.1#response",
  "threadId": "dd-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-12T09:00:01Z",
  "payload": { "entry": { "name": "tenant-a.example.com", "label": "Tenant A",
    "status": "disabled", "defaultDomain": false, "createdAt": "2026-06-10T09:00:01Z",
    "disabledAt": "2026-06-12T09:00:01Z", "purgeAt": "2026-06-19T09:00:01Z" } } }
```

## Security & Privacy

Admin-only. The transition cascades to read-only across every active DID on the domain — caller must understand the operational scope.
