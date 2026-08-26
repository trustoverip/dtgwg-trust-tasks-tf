---
slug: vta/webvh/dids/register-with-server
version: "1.0"
title: "VTA WebVH DIDs — Register With Server"
summary: "A super-administrator hands an existing DID's log to a hosting server so it is served there."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - hosting
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
  rationale: "Publishing a DID at a hosting location is visible to every party that resolves it, and can displace another registration. Attribution must not depend on the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Registration tells a witness server to serve this DID's log. A replayed registration reinstates a server the operator removed, and resolvers will then read the DID from it.
sideEffects:
  level: mutating
  rationale: "Publishes a DID's log at a hosting server; with force, may displace an existing registration."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/register-with-server:locationTaken"
    meaning: "The hosting location already holds a different registration and force was not set."
    retryable: false
related:
  - vta/webvh/servers/list
  - vta/webvh/servers/register
  - vta/webvh/dids/create
---

## Abstract

**VTA WebVH DIDs — Register With Server** hands an existing DID's log to a
hosting server, so the DID resolves there. It is how a serverless DID becomes
served, and how a DID moves to a different host.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** displace an existing
registration at the location unless `force` is true.

The response's `logEntryCount` is what the server accepted. A consumer **MUST**
compare it with the DID's own log length before reporting success: a shorter
count means the server holds a truncated history and will resolve to an older
document than the VTA believes is current.

## Authorization

Authority is the **super-administrator role**. It is higher than the
administrator role that creates a DID because this task publishes at a location
that may already belong to someone else's identity, and `force` makes that
displacement possible.

`force` is an explicitness flag, not a separate capability — a consumer **MUST
NOT** require a different role for it.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/register-with-server/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "serverId": "prod",
    "domain": "example.com",
    "force": false
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
  "type": "https://trusttasks.org/spec/vta/webvh/dids/register-with-server/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "serverId": "prod",
    "logEntryCount": 4
  }
}
```

## Security & Privacy

A DID's identity is bound to where it is served. Moving a non-portable DID to a
different domain does not carry its identity with it — the result is a different
DID that happens to share a history, and relying parties will not follow.

`force` overwrites what a hosting location already serves. Whoever was there
stops resolving, without being told.
