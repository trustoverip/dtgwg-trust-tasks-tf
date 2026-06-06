---
slug: did-management/domain/unassign
version: "0.1"
title: DID Management — Domain Unassign
summary: An administrator removes the binding between a hosting domain and a server instance — the instance stops serving DIDs on that domain after the ack round-trip.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, unassign, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service (control plane)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Unassignment changes fleet routing; record retention helps cross-instance debugging.
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `domain` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/unassign:unknown_instance
    meaning: The submitted `instanceId` does not match a known registry entry.
    retryable: false
related: [did-management/domain/assign]
---

## Abstract

The **Domain Unassign** Trust Task is the inverse of `domain/assign`. The control plane queues an unassign DIDComm message; on ack, the registry's `servedDomains` for that instance drops the pair. Until the ack arrives, the instance keeps serving DIDs on the domain — there is no synchronous cutover.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/unassign/0.1` with `payload.instanceId` and `payload.domain`. Consumer queues the outbound op and replies with the queued status.

## Request

```json
{ "id": "ua-1", "type": "https://trusttasks.org/spec/did-management/domain/unassign/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-17T09:00:00Z",
  "payload": { "instanceId": "did_web_node1_example", "domain": "tenant-a.example.com" } }
```

## Response

```json
{ "id": "ua-1-r", "type": "https://trusttasks.org/spec/did-management/domain/unassign/0.1#response",
  "threadId": "ua-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-17T09:00:01Z",
  "payload": { "instanceId": "did_web_node1_example", "domain": "tenant-a.example.com", "status": "queued" } }
```

## Security & Privacy

Admin-only. The threadId-linked ack from the unassigned instance completes the audit trail.
