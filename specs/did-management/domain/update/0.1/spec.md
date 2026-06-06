---
slug: did-management/domain/update
version: "0.1"
title: DID Management — Domain Update
summary: An administrator updates the metadata of an existing hosting domain (label only — the `name` itself is immutable).
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, update, admin]
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
  rationale: An update is an admin action with audit value.
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
related: [did-management/domain/create]
---

## Abstract

The **Domain Update** Trust Task mutates the metadata of an existing hosting domain. The `name` is immutable — to rename a domain, create a new one and migrate slots manually. Today only `label` is mutable; future spec versions MAY add additional mutable fields.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/update/0.1` with `payload.name` and `payload.label`. The consumer updates the entry and returns the new `DomainEntry`.

## Request

```json
{ "id": "du-1", "type": "https://trusttasks.org/spec/did-management/domain/update/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-11T09:00:00Z",
  "payload": { "name": "tenant-a.example.com", "label": "Tenant A (rebranded)" } }
```

## Response

```json
{ "id": "du-1-r", "type": "https://trusttasks.org/spec/did-management/domain/update/0.1#response",
  "threadId": "du-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-11T09:00:01Z",
  "payload": { "entry": { "name": "tenant-a.example.com", "label": "Tenant A (rebranded)",
    "status": "active", "defaultDomain": false, "createdAt": "2026-06-10T09:00:01Z" } } }
```

## Security & Privacy

Admin-only. Document is retained as audit evidence.
