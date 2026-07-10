---
slug: did-management/server/health
version: "0.1"
title: DID Management — Server Health
summary: A liveness probe between control plane and a registered server — replaces the legacy `health-ping`/`health-pong` pair with one task and `#response`.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, server, health, ping]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Health probe initiator
    requirement: REQUIRED
    member: issuer
  - role: Health probe responder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Health checks are routine and transport-authenticated; a proof becomes valuable only if a participant retains the probe history for SLA evidence.
sideEffects:
  level: none
  rationale: "Liveness probe between control plane and server."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related: [did-management/server/register]
---

## Abstract

The **Server Health** Trust Task is the framework-aligned replacement for the legacy `health-ping` / `health-pong` pair. The initiator sends one task referencing the target instance; the responder replies with the same task under the `#response` fragment, carrying its own `observedAt` timestamp. Routine: not evidentiary unless retained.

## Status of this Document

Draft.

## Conformance

Either direction MAY initiate. The producer emits `type: https://trusttasks.org/spec/did-management/server/health/0.1` with `payload.instanceId`. The consumer replies with `ok: true` and a timestamp.

## Request

```json
{ "id": "h-1", "type": "https://trusttasks.org/spec/did-management/server/health/0.1",
  "issuer": "did:web:control.example.com", "recipient": "did:web:node1.example.com",
  "issuedAt": "2026-06-21T09:00:00Z",
  "payload": { "instanceId": "did_web_node1_example_com" } }
```

## Response

```json
{ "id": "h-1-r", "type": "https://trusttasks.org/spec/did-management/server/health/0.1#response",
  "threadId": "h-1", "issuer": "did:web:node1.example.com", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-21T09:00:01Z",
  "payload": { "instanceId": "did_web_node1_example_com", "ok": true,
    "observedAt": "2026-06-21T09:00:01Z" } }
```

## Security & Privacy

A "probe" leaks instance reachability to the initiator. Hosts MAY rate-limit health checks from non-registered senders.
