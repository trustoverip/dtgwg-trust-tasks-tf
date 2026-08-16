---
slug: did-management/registry/admin-register
version: "0.1"
title: DID Management — Registry Admin Register
summary: An administrator manually inserts a server instance into the control-plane registry — used to seed a known instance from configuration before the instance has had a chance to self-register.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, registry, admin, seed]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Control plane
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Admin-register is privileged — a forged record could let an attacker direct fleet messages at an instance they control. The maintainer SHOULD retain a signed authorisation.
sideEffects:
  level: mutating
  rationale: "Inserts a server instance into the control-plane registry; reversible via deregister."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/registry/admin-register:instance_exists
    meaning: A registry entry with the same `instanceId` is already present.
    retryable: false
related: [did-management/registry/deregister, did-management/server/register]
---

## Abstract

The **Registry Admin Register** Trust Task is the admin's hand-seed path. Distinct from `server/register/0.1` (the Service-role *server* announcing itself), this task lets an admin pre-populate a registry entry from configuration so the control plane knows about the instance before it ever connects. The seeded entry is updated by subsequent `server/register/0.1` calls when the instance comes online.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/registry/admin-register/0.1` with the entry fields. Consumer rejects duplicates and otherwise commits the entry.

## Authorization

*Stated in anticipation of [SPEC §7.3](../../../../../SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer** — the slug says `admin-register` precisely because the ordinary registration path is a different task with different authority.

The duplicate check Conformance names is an integrity constraint on the registry, not an authorization check; a caller without standing is refused with `permissionDenied` whether or not the entry already exists.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](../../../../../SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "ar-1", "type": "https://trusttasks.org/spec/did-management/registry/admin-register/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-23T09:00:00Z",
  "payload": { "instanceId": "did_web_node2_example_com",
    "did": "did:web:node2.example.com",
    "publicUrl": "https://node2.example.com",
    "servedDomains": [ "tenant-b.example.com" ],
    "label": "EU-West edge node 2" } }
```

## Response

```json
{ "id": "ar-1-r", "type": "https://trusttasks.org/spec/did-management/registry/admin-register/0.1#response",
  "threadId": "ar-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-23T09:00:01Z",
  "payload": { "entry": { "instanceId": "did_web_node2_example_com",
    "did": "did:web:node2.example.com", "publicUrl": "https://node2.example.com",
    "servedDomains": [ "tenant-b.example.com" ], "label": "EU-West edge node 2" } } }
```

## Security & Privacy

Admin-only. Misuse would let an attacker direct outbound fleet messages at an instance they control. The signed document and the receiving consumer's audit log form the trail.
