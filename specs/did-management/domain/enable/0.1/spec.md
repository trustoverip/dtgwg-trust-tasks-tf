---
slug: did-management/domain/enable
version: "0.1"
title: DID Management — Domain Enable
summary: An administrator re-enables a previously-disabled hosting domain, cancelling any pending purge and restoring it to active service.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, enable, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Re-enabling a domain has a fleet-wide effect symmetric to `domain/disable`; the maintainer SHOULD retain a signed record.
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/enable:already_purged
    meaning: The domain was already purged after its grace period expired; it can no longer be re-enabled and must be recreated via `domain/create`.
    retryable: false
related: [did-management/domain/disable, did-management/domain/purge, did-management/domain/create]
---

## Abstract

The **Domain Enable** Trust Task is the inverse of [`domain/disable`](../../disable/0.1/spec.md). It moves a disabled hosting domain back into active service, clearing the `disabledAt` and `purgeAt` markers and cancelling any pending purge that had been scheduled. New DID registrations under the domain resume immediately.

The operation is **idempotent** — re-enabling an already-active domain is a no-op and returns the current entry. A domain that has already been purged after its grace period cannot be re-enabled; the operator must recreate it via [`domain/create`](../../create/0.1/spec.md), which mints a fresh entry under the same name.

A successful enable triggers the same fleet-wide fan-out as create/update/disable: the control plane pushes the updated `DomainEntry` to every registered server so their resolvers see the active state without waiting for a refresh.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/enable/0.1` with `payload.name`. Consumer rejects with `did-management:unknown_domain` if the domain is not in the registry, and with `did-management/domain/enable:already_purged` if the domain was purged after its grace period. Otherwise the consumer clears `disabledAt`/`purgeAt`, sets `status: "active"`, fans out the updated entry to registered servers, and replies with the updated `DomainEntry`.

## Request

```json
{ "id": "de-1", "type": "https://trusttasks.org/spec/did-management/domain/enable/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-15T09:00:00Z", "payload": { "name": "tenant-a.example.com" } }
```

## Response

```json
{ "id": "de-1-r", "type": "https://trusttasks.org/spec/did-management/domain/enable/0.1#response",
  "threadId": "de-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-15T09:00:01Z",
  "payload": { "entry": { "name": "tenant-a.example.com", "label": "Tenant A",
    "status": "active", "defaultDomain": false, "createdAt": "2026-06-10T09:00:01Z" } } }
```

The response carries the canonical `DomainEntry` after re-enabling — `disabledAt` and `purgeAt` are absent on an active entry.

## Security & Privacy

Admin-only. Enabling a domain restores the path-namespace it owned before disable; consumers MUST ensure the caller still holds admin authority on the domain and that no conflicting `domain/create` has bound the same `name` to a different entry during the grace window.
