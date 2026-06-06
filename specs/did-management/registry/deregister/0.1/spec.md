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
errorCodes:
  - code: did-management/registry/deregister:not_found
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
