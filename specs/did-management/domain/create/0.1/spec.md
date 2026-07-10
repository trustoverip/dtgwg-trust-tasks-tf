---
slug: did-management/domain/create
version: "0.1"
title: DID Management — Domain Create
summary: An administrator adds a new hosting domain to a DID hosting service.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, create, admin]
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
  rationale: Adding a hosting domain enables every downstream operation under that domain. The maintainer SHOULD retain a signed record of who created the domain.
sideEffects:
  level: mutating
  rationale: "Adds a hosting domain; reversible via disable/purge."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/domain/create:domain_exists
    meaning: A domain with the same `name` already exists.
    retryable: false
  - code: did-management/domain/create:invalid_name
    meaning: The submitted `name` violates the host's hostname grammar.
    retryable: false
related: [did-management/domain/update, did-management/domain/disable, did-management/domain/set-default]
---

## Abstract

The **Domain Create** Trust Task adds a new hosting domain to the service. Once created, the domain becomes a permissible value for the `domain` field in every DID-management op (see [category conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-domain-resolution)). The optional `setAsDefault` flag makes the new domain the system default in the same atomic step — useful when bootstrapping.

## Status of this Document

Draft.

## Conformance

The producer (admin) emits `type: https://trusttasks.org/spec/did-management/domain/create/0.1` with `payload.name`. The consumer verifies the caller has admin authority, validates the name, and commits the new `DomainEntry`.

## Request

```json
{ "id": "dc-1", "type": "https://trusttasks.org/spec/did-management/domain/create/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-10T09:00:00Z",
  "payload": { "name": "tenant-a.example.com", "label": "Tenant A", "setAsDefault": false } }
```

## Response

```json
{ "id": "dc-1-r", "type": "https://trusttasks.org/spec/did-management/domain/create/0.1#response",
  "threadId": "dc-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-10T09:00:01Z",
  "payload": { "entry": { "name": "tenant-a.example.com", "label": "Tenant A",
    "status": "active", "defaultDomain": false, "createdAt": "2026-06-10T09:00:01Z" } } }
```

## Security & Privacy

Admin-only. The signed document is the evidentiary record of the domain's introduction.
