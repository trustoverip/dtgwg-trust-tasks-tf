---
slug: did-management/domain/set-default
version: "0.1"
title: DID Management — Domain Set Default
summary: An administrator promotes a hosting domain to be the system default — every operation that omits an explicit domain resolves to this one (subject to the per-caller ACL default tier in front of it).
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, set-default, admin]
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
  rationale: Moving the system default changes the answer to every "no domain specified" operation; an evidentiary record protects against later disputes about when the cutover took effect.
sideEffects:
  level: mutating
  rationale: "Promotes a domain to the system default; reversible by setting another."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/set-default:domain_disabled
    meaning: The domain is in the `disabled` state and cannot be promoted to default.
    retryable: false
related: [did-management/domain/create, did-management/domain/disable]
---

## Abstract

The **Domain Set Default** Trust Task promotes one hosting domain to the system default. Exactly one domain at any time carries `defaultDomain: true` — promoting a new default atomically clears the prior one. The response carries both the new and prior default for confirmation.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/set-default/0.1` with `payload.name`. Consumer atomically swaps the default flag and returns the new entry plus the prior default name (`null` if there was no prior default).

## Authorization

*Stated in anticipation of [SPEC §7.3](../../../../../SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer**. Changing which domain is the default changes where subsequent operations land by default, so the consumer establishes standing from the authenticated producer identity and its own records rather than from anything the document asserts.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](../../../../../SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "sd-1", "type": "https://trusttasks.org/spec/did-management/domain/set-default/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-13T09:00:00Z", "payload": { "name": "tenant-b.example.com" } }
```

## Response

```json
{ "id": "sd-1-r", "type": "https://trusttasks.org/spec/did-management/domain/set-default/0.1#response",
  "threadId": "sd-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-13T09:00:01Z",
  "payload": { "entry": { "name": "tenant-b.example.com", "status": "active",
    "defaultDomain": true, "createdAt": "2026-06-09T08:00:00Z" },
    "previousDefault": "tenant-a.example.com" } }
```

## Security & Privacy

Admin-only. Audit trail value rather than confidentiality risk.
