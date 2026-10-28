---
slug: did-management/registry/admin-register
version: "0.2"
title: DID Management — Registry Admin Register
summary: An administrator manually inserts a server instance into the control-plane registry — used to seed a known instance from configuration before the instance has had a chance to self-register.
status: draft
targetFrameworkVersion: "0.5.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An administrative registration bypasses the ordinary claim path, so a captured one is a standing ability to register on the registry’s own authority until the acceptance window closes it.
sideEffects:
  level: mutating
  rationale: "Inserts a server instance into the control-plane registry; reversible via deregister."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/registry/admin-register:instanceExists
    meaning: A registry entry with the same `instanceId` is already present.
    retryable: false
related: [did-management/registry/deregister, did-management/server/register]
---

## Abstract

The **Registry Admin Register** Trust Task is the admin's hand-seed path. Distinct from `server/register/0.1` (the Service-role *server* announcing itself), this task lets an admin pre-populate a registry entry from configuration so the control plane knows about the instance before it ever connects. The seeded entry is updated by subsequent `server/register/0.1` calls when the instance comes online.

## Status of this Document

Draft.

## Changes from 0.1

* The response `entry` (shared `ServiceInstance`) member **`lastSeen`** → **`lastSeenAt`**. Same type (`string`, `date-time`) and meaning.

The rename brings this member into line with the registry's naming convention — an instant that records an event is named `<event>At` — so the same concept has one spelling across the registry. The response's `$ref` moves from `did-management/_shared/0.1/service-instance` to `did-management/_shared/0.3/registered-instance`, which is 0.1's `ServiceInstance` with only this rename. It is deliberately not `_shared/0.3/service-instance`, the richer record the registry reads return: re-pinning to that would add required members to the response, a change beyond the rename. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/registry/admin-register/0.2` with the entry fields. Consumer rejects duplicates and otherwise commits the entry.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer** — the slug says `admin-register` precisely because the ordinary registration path is a different task with different authority.

The duplicate check Conformance names is an integrity constraint on the registry, not an authorization check; a caller without standing is refused with `permissionDenied` whether or not the entry already exists.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "ar-1", "type": "https://trusttasks.org/spec/did-management/registry/admin-register/0.2",
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
{ "id": "ar-1-r", "type": "https://trusttasks.org/spec/did-management/registry/admin-register/0.2#response",
  "threadId": "ar-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-23T09:00:01Z",
  "payload": { "entry": { "instanceId": "did_web_node2_example_com",
    "did": "did:web:node2.example.com", "publicUrl": "https://node2.example.com",
    "servedDomains": [ "tenant-b.example.com" ], "label": "EU-West edge node 2" } } }
```

## Security & Privacy

### Data carried

The request carries the seeded instance's identifier, DID, public URL, and optionally its served hosting domains and a display label; the response echoes them as the stored registry entry, with `lastSeenAt` once the instance has been seen. No key material is carried.

### Correlation

Every identifier the payload carries names a hosting service rather than a person, and is the one the control plane will address that service by. This specification imposes no further rule.

### Retention

The signed document and the receiving consumer's audit log form the trail.

### Consent/purpose

Admin-only. Misuse would let an attacker direct outbound fleet messages at an instance they control.
