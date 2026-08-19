---
slug: vta/webvh/servers/register
version: "1.0"
title: "VTA WebVH Servers — Register"
summary: "A super-administrator registers a did:webvh hosting server with the VTA."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - hosting
  - server
  - register
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
  rationale: "A hosting server serves the VTA's DIDs to the world; registering the wrong one redirects identities. Attribution must not depend on the transport."
sideEffects:
  level: mutating
  rationale: "Adds a hosting registration the VTA will publish DIDs through."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/servers/register:idConflict"
    meaning: "A server is already registered under this id."
    retryable: false
related:
  - vta/webvh/servers/list
  - vta/webvh/servers/remove
  - vta/webvh/dids/register-with-server
---

## Abstract

**VTA WebVH Servers — Register** adds a hosting server the VTA can publish DIDs
through, under a local id that DIDs then reference.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse an id already in use with
`vta/webvh/servers/register:idConflict` rather than replacing the existing
registration — DIDs reference servers by this id, and silently repointing it
would move every one of them.

## Authorization

Authority is the **super-administrator role**. A hosting server is where this
VTA's identities are published from, and trusting the wrong one hands an
attacker the ability to serve documents in the VTA's name.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](../../../../../../SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/register/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "id": "prod",
    "did": "did:web:daemon.example",
    "label": "prod hosting"
  },
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
  "type": "https://trusttasks.org/spec/vta/webvh/servers/register/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "id": "prod",
    "did": "did:web:daemon.example",
    "label": "prod hosting",
    "createdAt": "2026-08-19T11:00:01Z",
    "updatedAt": "2026-08-19T11:00:01Z"
  }
}
```

## Security & Privacy

The server's `did` is what the VTA authenticates against when it publishes.
Registering a DID the operator has not verified out of band means the VTA will
faithfully hand its DID logs to whoever controls it.
