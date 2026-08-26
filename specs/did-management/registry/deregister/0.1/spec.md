---
slug: did-management/registry/deregister
version: "0.1"
title: DID Management — Registry Deregister
summary: An administrator removes a server instance from the control-plane registry.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, registry, admin, deregister]
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
  rationale: Deregister cuts the instance off from outbound fleet messages. The maintainer SHOULD retain a signed record of the change.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deregistration removes a DID from the registry other parties resolve against. Replayed after re-registration it removes the new entry, and the registry has nothing but this timestamp to date the instruction by.
sideEffects:
  level: mutating
  rationale: "Removes a server instance from the control-plane registry; the instance may re-register."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/registry/deregister:notFound
    meaning: No registry entry matches the submitted `instanceId`.
    retryable: false
related: [did-management/registry/admin-register, did-management/server/register]
---

## Abstract

The **Registry Deregister** Trust Task removes a server instance from the registry. After the operation, the control plane no longer sends domain-assign / sync / health messages to that instance. The instance MAY still self-rejoin via `server/register/0.1` — deregister is reversible by re-onboarding.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/registry/deregister/0.1` with `payload.instanceId`. Consumer verifies the entry exists and removes it.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer**, established from the authenticated producer identity and the consumer's own records.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "dr-1", "type": "https://trusttasks.org/spec/did-management/registry/deregister/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-24T09:00:00Z", "payload": { "instanceId": "did_web_node2_example_com" } }
```

## Response

```json
{ "id": "dr-1-r", "type": "https://trusttasks.org/spec/did-management/registry/deregister/0.1#response",
  "threadId": "dr-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-24T09:00:01Z",
  "payload": { "instanceId": "did_web_node2_example_com",
    "removedAt": "2026-06-24T09:00:01Z" } }
```

## Security & Privacy

Admin-only. The deregistered instance's `servedDomains` claims are no longer reflected in the control plane's routing view; operators MUST take care not to deregister a still-running instance that's the sole holder of a hosted domain.
