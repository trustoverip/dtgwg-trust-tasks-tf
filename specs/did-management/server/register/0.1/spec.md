---
slug: did-management/server/register
version: "0.1"
title: DID Management — Server Register
summary: A hosting server announces itself to the control plane, declaring which hosting domains it serves and where it can be reached.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, server, register, fleet]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Hosting server (Service role)
    requirement: REQUIRED
  - role: Control plane
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: A register is a fleet-membership announcement. The maintainer authenticates the producer via the transport-layer Service-role binding; a proof becomes valuable when the registration is replayed for audit but isn't strictly required.
errorCodes:
  - code: did-management/server/register:forbidden
    meaning: Caller does not hold the `Service` role on this control plane.
    retryable: false
related: [did-management/domain/assign, did-management/server/health]
---

## Abstract

The **Server Register** Trust Task is how a hosting server tells the control plane "I'm online, here's my reachable URL, here are the hosting domains I'm currently serving." The control plane updates the registry entry keyed by the server's stable `instanceId` (typically derived from the server's DID). This is also the trigger that causes the control plane to push the server every DID log + witness content for the domains it serves — so the call MUST be Service-role only, not open to any authenticated caller.

## Status of this Document

Draft.

## Conformance

A conforming producer (the hosting server) MUST hold the `Service` role on the control plane's ACL. Emits `type: https://trusttasks.org/spec/did-management/server/register/0.1` with `payload.instanceId`, `payload.did`, `payload.publicUrl`, and `payload.servedDomains[]`. The consumer (control plane) verifies the Service-role binding, upserts the registry entry, and replies with the accepted flag plus the `lastSeen` timestamp it stamped.

The producer MAY additionally declare its capabilities:

- `payload.enabledMethods[]` — the DID methods the server is willing to host (e.g. `["webvh", "web"]`). Consumers SHOULD record this against the registry entry so the operator can see which methods each server supports and route domain assignments accordingly. When omitted, consumers MUST assume only `webvh` is supported (the historical default).
- `payload.protocolVersion` — the wire-protocol revision the server speaks (e.g. `"1.0"`). Consumers SHOULD record this to support future protocol-level negotiation. When omitted, consumers MUST assume `"1.0"`.

Both fields are additive and ignorable by older consumers; producers that omit them remain conformant.

## Request

```json
{ "id": "sr-1", "type": "https://trusttasks.org/spec/did-management/server/register/0.1",
  "issuer": "did:web:node1.example.com", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-20T09:00:00Z",
  "payload": { "instanceId": "did_web_node1_example_com", "did": "did:web:node1.example.com",
    "publicUrl": "https://node1.example.com", "servedDomains": ["tenant-a.example.com"],
    "label": "EU-West edge node 1",
    "enabledMethods": ["webvh"], "protocolVersion": "1.0" } }
```

## Response

```json
{ "id": "sr-1-r", "type": "https://trusttasks.org/spec/did-management/server/register/0.1#response",
  "threadId": "sr-1", "issuer": "did:web:control.example.com", "recipient": "did:web:node1.example.com",
  "issuedAt": "2026-06-20T09:00:01Z",
  "payload": { "instanceId": "did_web_node1_example_com", "accepted": true,
    "lastSeen": "2026-06-20T09:00:01Z" } }
```

## Security & Privacy

The Service-role gate is load-bearing: a successful register triggers a full DID-content sync to the registering instance. Non-Service callers MUST be rejected with `did-management/server/register:forbidden`.
