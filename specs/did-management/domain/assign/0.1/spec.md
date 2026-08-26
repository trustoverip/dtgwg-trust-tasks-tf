---
slug: did-management/domain/assign
version: "0.1"
title: DID Management — Domain Assign
summary: An administrator binds a hosting domain to a registered server instance — the bound server commits to serving DIDs on that domain.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, assign, admin]
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
  rationale: Assignment shapes the fleet's traffic routing; an evidentiary record is valuable for cross-instance debugging.
sideEffects:
  level: mutating
  rationale: "Binds a hosting domain to a server instance; reversible via unassign."
subjectPath: /domain
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknownDomain
    meaning: The submitted `domain` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/assign:unknownInstance
    meaning: The submitted `instanceId` does not match a known registry entry.
    retryable: false
related: [did-management/domain/unassign, did-management/server/register]
---

## Abstract

The **Domain Assign** Trust Task pairs a hosting domain with a server instance in the host's registry. The control plane queues a `domain.assign/1.0` DIDComm message to the bound instance; the instance acks via the same channel, at which point the registry's `servedDomains` includes the new pair. The response carries `status: "queued"` immediately — fully synchronous binding is not guaranteed.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/assign/0.1` with `payload.instanceId` and `payload.domain`. Consumer validates both, queues the outbound assign, and replies with the queued status.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer**, in the sense the consumer's own deployment defines. The Abstract calls the producer an "admin caller"; that names who the task expects, not something the document proves.

Nothing in the payload carries that standing, and nothing in the document can. The consumer establishes it from the authenticated producer identity and its own records, and refuses with `permissionDenied` where it is absent.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "da-1", "type": "https://trusttasks.org/spec/did-management/domain/assign/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-16T09:00:00Z",
  "payload": { "instanceId": "did_web_node1_example", "domain": "tenant-a.example.com" } }
```

## Response

```json
{ "id": "da-1-r", "type": "https://trusttasks.org/spec/did-management/domain/assign/0.1#response",
  "threadId": "da-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-16T09:00:01Z",
  "payload": { "instanceId": "did_web_node1_example", "domain": "tenant-a.example.com", "status": "queued" } }
```

## Security & Privacy

Admin-only. The audit trail crosses instances: the control plane retains this document, and the instance's ack carries the same `threadId`.
