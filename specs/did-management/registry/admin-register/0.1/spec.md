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
