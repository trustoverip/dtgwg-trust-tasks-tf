---
slug: vta/webvh/servers/remove
version: "1.0"
title: "VTA WebVH Servers — Remove"
summary: "A super-administrator removes a hosting registration; DIDs the server already serves keep resolving."
status: draft
targetFrameworkVersion: "0.5"
category: did-management
keywords:
  - vta
  - webvh
  - hosting
  - server
  - remove
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Super-administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Removing a registration severs the VTA's route for updating every DID published through it. Attribution must not depend on the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Removing a server stops it serving the operator’s logs. Replayed after a server was re-registered it removes the new registration, and can leave DIDs with nothing publishing them.
sideEffects:
  level: mutating
  rationale: "Removes the registration. Published DIDs are unaffected and keep resolving."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/servers/remove:notFound"
    meaning: "No server is registered under this id."
    retryable: false
related:
  - vta/webvh/servers/list
  - vta/webvh/servers/register
  - vta/webvh/dids/list
---

## Abstract

**VTA WebVH Servers — Remove** deletes a hosting registration.

It does **not** unpublish anything. DIDs the server already serves keep
resolving exactly as before; what the VTA loses is its route for updating
them.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** delete, unpublish or modify
any DID as a consequence of this task, and **MUST NOT** represent the removal as
having done so.

`removed: false` in a success response means the VTA declined to act.

## Authorization

Authority is the **super-administrator role**, matching the registration it
undoes.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/remove/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": { "id": "prod" },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T11:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/remove/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": { "id": "prod", "removed": true }
}
```

## Security & Privacy

The DIDs left behind are the risk. They continue to resolve from a server the
VTA no longer tracks, so their documents can no longer be rotated or corrected
through it — and an operator reading a list of their DIDs afterwards will see
entries whose hosting the VTA can no longer reach.
