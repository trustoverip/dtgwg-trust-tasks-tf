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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Unassignment strips a domain from its owner and, replayed after a reassignment, strips it from the new owner instead. The document names the domain but not the assignment it was written about, so the timestamp is what scopes it.
sideEffects:
  level: mutating
  rationale: "Removes a domain-to-server binding; reversible via assign."
subjectPath: /domain
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknownDomain
    meaning: The submitted `domain` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/unassign:unknownInstance
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

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer**, established from the authenticated producer identity and the consumer's own records. Nothing in the payload conveys it.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

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
