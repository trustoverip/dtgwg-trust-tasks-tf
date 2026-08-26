---
slug: vta/webvh/servers/list
version: "1.0"
title: "VTA WebVH Servers — List"
summary: "Enumerate the did:webvh hosting servers a VTA is registered with."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - hosting
  - server
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "A read with no durable state change."
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/webvh/servers/register
  - vta/webvh/servers/remove
  - vta/webvh/dids/list
---

## Abstract

**VTA WebVH Servers — List** returns the hosting servers this VTA is registered
with — the services that serve its DIDs' logs.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** include any credential,
token or secret it holds for authenticating to a server. The record describes
the registration; it is not a way to read it back out.

## Authorization

Any authenticated caller may read this. Hosting registrations are operational
topology rather than authority: knowing which servers a VTA publishes through
confers nothing, and the DIDs themselves are already public where they are
served.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/list/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "servers": [
      {
        "id": "prod",
        "did": "did:web:daemon.example",
        "label": "prod hosting",
        "createdAt": "2026-01-04T10:00:00Z",
        "updatedAt": "2026-01-04T10:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

The exclusion of credentials from this record is normative rather than
incidental: a hosting registration is the one place a VTA holds a long-lived
secret for a third-party service, and a list endpoint that returned it would
turn a read capability into an impersonation capability.
